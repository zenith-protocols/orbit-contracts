use crate::storage;
use crate::dependencies::pool::{Client as PoolClient, Request};
use crate::dependencies::pool_factory::{Client as PoolFactoryClient};
use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, vec, Address, BytesN, Env, IntoVal, Symbol, TryFromVal, Val, Vec};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use crate::errors::TreasuryError;

const REQUEST_TYPE_SUPPLY: u32 = 0;
const REQUEST_TYPE_WITHDRAW: u32 = 1;

#[contract]
pub struct TreasuryContract;

#[contractclient(name="TreasuryClient")]
pub trait Treasury {

    /// (Admin only) add a stablecoin
    ///
    /// ### Arguments
    /// * `token` - The Address for the token
    /// * `blend_pool` - The Address for the blend pool
    ///
    /// ### Panics
    /// If the caller is not the dao-utils
    fn add_stablecoin(e: Env, token: Address, blend_pool: Address);

    /// (Admin only) Increase the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to increase the supply by
    ///
    /// ### Panics
    /// If the caller is not the dao-utils
    fn increase_supply(e: Env, token: Address, amount: i128);

    /// (Admin only) Decrease the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to decrease the supply by
    ///
    /// ### Panics
    /// If the caller is not the dao-utils
    /// If the supply is less than the amount
    fn decrease_supply(e: Env, token: Address, amount: i128);

    /// (Admin only) Claim interest from the blend pool
    ///
    /// ### Arguments
    /// * `pool` - The blend pool to claim interest from
    /// * `reserve_tokens_id` - The reserve tokens id of the tokens to claim interest from
    /// * `to` - The address to send the interest to
    ///
    /// ### Panics
    /// If the caller is not the dao-utils
    fn claim_interest(e: Env, pool: Address, reserve_tokens_id: Vec<u32>, to: Address) -> i128;

    /// Flashloan function for keeping the peg of stablecoins
    ///
    /// ### Arguments
    /// * `name` - The name of the function to call
    /// * `args` - The arguments for the function first argument has to be token and second amount
    ///
    /// ### Panics
    /// If the flashloan is not profitable
    fn keep_peg(e: Env, name: Symbol, args: Vec<Val>);

    /// (Admin only) Set a new address as the pegkeeper
    ///
    /// ### Arguments
    /// * `pegkeeper` - The new pegkeeper address
    ///
    /// ### Panics
    /// If the caller is not the dao-utils
    fn set_pegkeeper(e: Env, pegkeeper: Address);

    /// Updates this contract to a new version
    /// # Arguments
    /// * `new_wasm_hash` - The new wasm hash
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
}

impl TreasuryContract {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `dao-utils` - The Address for the dao-utils
    /// * `factory` - The Address for the blend factory
    /// * `pegkeeper` - The Address for the pegkeeper
    ///
    /// ### Panics
    /// If the contract is already initialized
    fn __constructor(e: Env, admin: Address, factory: Address, pegkeeper: Address) {
        admin.require_auth();

        storage::set_pegkeeper(&e, &pegkeeper);
        storage::set_factory(&e, &factory);
        storage::set_admin(&e, &admin);

        e.events().publish(("Treasury", Symbol::new(&e, "initialize")), (admin.clone(), pegkeeper.clone()));
    }
}

#[contractimpl]
impl Treasury for TreasuryContract {

    fn add_stablecoin(e: Env, token: Address, blend_pool: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if let Some(_) = storage::get_blend_pool(&e, &token) {
            panic_with_error!(e, TreasuryError::AlreadyAddedError);
        }

        let is_pool = PoolFactoryClient::new(&e, &storage::get_factory(&e)).is_pool(&blend_pool);
        if !is_pool {
            panic_with_error!(e, TreasuryError::InvalidBlendPoolError);
        }

        storage::set_blend_pool(&e, &token, &blend_pool);

        e.events().publish(("Treasury", Symbol::new(&e, "add_stablecoin")), (token.clone(), blend_pool.clone()));
    }

    fn increase_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if amount <= 0 {
            panic_with_error!(e, TreasuryError::InvalidAmount);
        }

        token::StellarAssetClient::new(&e, &token).mint(&e.current_contract_address(), &amount);

        let blend = storage::get_blend_pool(&e, &token).unwrap_or_else(|| {
            panic_with_error!(e, TreasuryError::BlendPoolNotFoundError);
        });
        let args: Vec<Val> = vec![
            &e,
            e.current_contract_address().into_val(&e),
            blend.into_val(&e),
            amount.into_val(&e),
        ];
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: args.clone(),
                },
                sub_invocations: vec![&e],
            })
        ]);
        PoolClient::new(&e, &blend).submit(&e.current_contract_address(), &e.current_contract_address(), &e.current_contract_address(), &vec![
            &e,
            Request {
                request_type: REQUEST_TYPE_SUPPLY,
                address: token.clone(),
                amount,
            },
        ]);

        e.events().publish(("Treasury", Symbol::new(&e, "increase_supply")), (token.clone(), amount.clone()));
    }

    fn decrease_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if amount <= 0 {
            panic_with_error!(e, TreasuryError::InvalidAmount);
        }

        let token_client = token::TokenClient::new(&e, &token);
        let balance = token_client.balance(&e.current_contract_address());

        let blend = storage::get_blend_pool(&e, &token).unwrap_or_else(|| {
            panic_with_error!(e, TreasuryError::BlendPoolNotFoundError);
        });
        PoolClient::new(&e, &blend).submit(&e.current_contract_address(), &e.current_contract_address(), &e.current_contract_address(), &vec![
            &e,
            Request {
                request_type: REQUEST_TYPE_WITHDRAW,
                address: token.clone(),
                amount,
            },
        ]);

        let balance_after = token_client.balance(&e.current_contract_address());
        if (balance_after - balance) < amount {
            panic_with_error!(e, TreasuryError::NotEnoughSupplyError);
        }

        token::TokenClient::new(&e, &token).burn(&e.current_contract_address(), &amount);
        e.events().publish(("Treasury", Symbol::new(&e, "decrease_supply")), (token.clone(), amount.clone()));
    }

    fn claim_interest(e: Env, pool: Address, reserve_tokens_id: Vec<u32>, to: Address) -> i128 {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let tokens_claimed = PoolClient::new(&e, &pool).claim(&e.current_contract_address(), &reserve_tokens_id, &to);

        e.events().publish(("Treasury", Symbol::new(&e, "claim_interest")), (pool.clone(), reserve_tokens_id.clone(), to.clone(), tokens_claimed.clone()));

        tokens_claimed
    }

    fn keep_peg(e: Env, name: Symbol, args: Vec<Val>) {
        storage::extend_instance(&e);

        let token = Address::try_from_val(&e, &args.get(0).unwrap()).unwrap();
        let amount = i128::try_from_val(&e, &args.get(1).unwrap()).unwrap();
        let pool = Address::try_from_val(&e, &args.get(2).unwrap()).unwrap();
        let pegkeeper: Address = storage::get_pegkeeper(&e);

        if amount <= 0 {
            panic_with_error!(e, TreasuryError::InvalidAmount);
        }
        let blend = storage::get_blend_pool(&e, &token).unwrap_or_else(|| {
            panic_with_error!(e, TreasuryError::BlendPoolNotFoundError);
        });
        if blend != pool {
            panic_with_error!(e, TreasuryError::InvalidBlendPoolError);
        }

        token::StellarAssetClient::new(&e, &token).mint(&pegkeeper, &amount);

        let token_client = token::TokenClient::new(&e, &token);

        e.invoke_contract::<Val>(&pegkeeper, &name, args.clone());

        let res = token_client.try_transfer_from(
            &e.current_contract_address(),
            &pegkeeper,
            &e.current_contract_address(),
            &amount,
        );

        if let Ok(Ok(_)) = res {
            token_client.burn(&e.current_contract_address(), &amount);
        } else {
            panic_with_error!(e, TreasuryError::FlashloanFailedError);
        }

        e.events().publish(("Treasury", Symbol::new(&e, "keep_peg")), (token.clone(), amount.clone()));
    }

    fn set_pegkeeper(e: Env, new_pegkeeper: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_pegkeeper(&e, &new_pegkeeper);

        e.events().publish(("Treasury", Symbol::new(&e, "set_pegkeeper")), new_pegkeeper.clone());
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
