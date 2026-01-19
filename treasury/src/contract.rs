use crate::storage;
use crate::dependencies::pool::{Client as PoolClient, Request};
use crate::dependencies::pool_factory::{Client as PoolFactoryClient};
use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, vec, Address, Env, IntoVal, Symbol, TryFromVal, Val, Vec};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use soroban_fixed_point_math::{i128, FixedPoint};
use crate::constants::{REQUEST_TYPE_SUPPLY, REQUEST_TYPE_WITHDRAW, SCALAR_12};
use crate::errors::TreasuryError;

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
    /// If the caller is not the admin
    fn add_stablecoin(e: Env, token: Address, blend_pool: Address);

    /// (Admin only) Increase the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to increase the supply by
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn increase_supply(e: Env, token: Address, amount: i128);

    /// (Admin only) Decrease the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to decrease the supply by
    ///
    /// ### Panics
    /// If the caller is not the admin
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
    /// If the caller is not the admin
    fn claim(e: Env, reserve_address: Address, to: Address) -> i128;

    /// (Admin only) Set a new address as the admin
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    fn set_admin(e: Env, new_admin: Address);
}

#[contractimpl]
impl TreasuryContract {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `admin` - The Address of the admin
    /// * `factory` - The Address for the blend factory
    ///
    /// ### Panics
    /// If the contract is already initialized
    pub fn __constructor(e: Env, admin: Address, factory: Address) {
        admin.require_auth();

        storage::set_factory(&e, &factory);
        storage::set_admin(&e, &admin);

        e.events().publish(("Treasury", Symbol::new(&e, "initialize")), (admin.clone(),));
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
                amount: amount.clone(),
            },
        ]);
        
        let mut total_supply = storage::get_total_supply(&e, &token.clone());
        total_supply += amount as i128;
        storage::set_total_supply(&e, &token.clone(), &total_supply);

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

        let mut total_supply = storage::get_total_supply(&e, &token.clone());
        total_supply -= amount as i128;
        storage::set_total_supply(&e, &token.clone(), &total_supply);

        token::TokenClient::new(&e, &token).burn(&e.current_contract_address(), &amount);
        e.events().publish(("Treasury", Symbol::new(&e, "decrease_supply")), (token.clone(), amount.clone()));
    }

    fn claim(e: Env, reserve_address: Address, to: Address) -> i128 {
        storage::extend_instance(&e);
    
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let blend_pool = storage::get_blend_pool(&e, &reserve_address).unwrap_or_else(|| {
            panic_with_error!(e, TreasuryError::BlendPoolNotFoundError);
        });
        let pool_client = PoolClient::new(&e, &blend_pool);
        let reserve = pool_client.get_reserve(&reserve_address);
        let b_rate = reserve.data.b_rate;
        let position = pool_client.get_positions(&e.current_contract_address());

        let b_token = position.supply.get(reserve.config.index).unwrap();
        let underlying = b_token.fixed_mul_floor(b_rate, SCALAR_12).unwrap();
        let interest = underlying - storage::get_total_supply(&e, &reserve_address.clone());

        if interest <= 0 {
            panic_with_error!(e, TreasuryError::NoInterestToClaim);
        }

        pool_client.submit(&e.current_contract_address(), &e.current_contract_address(), &to, &vec![
            &e,
            Request {
                request_type: REQUEST_TYPE_WITHDRAW,
                address: reserve_address.clone(),
                amount: interest,
            },
        ]);

        e.events().publish(("Treasury", Symbol::new(&e, "claim")), (reserve_address.clone(), to.clone(), interest.clone()));
        interest
    }

    fn set_admin(e: Env, new_admin: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_admin(&e, &new_admin);

        e.events().publish(("Treasury", Symbol::new(&e, "set_admin")), new_admin.clone(),);
    }
}
