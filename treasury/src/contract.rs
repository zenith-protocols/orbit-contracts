use crate::storage;
use crate::dependencies::pool::{Client as PoolClient, Request};
use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, vec, Address, Env, IntoVal, Symbol, Val, Vec};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use crate::errors::TreasuryError;
use token::Client as TokenClient;
use sep_41_token::StellarAssetClient;

#[contract]
pub struct TreasuryContract;

#[contractclient(name="TreasuryClient")]
pub trait Treasury {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `admin` - The Address for the admin
    /// * `token` - The Address for the token
    /// * `blend_pool` - The Address for the blend pool
    ///
    /// ### Panics
    /// If the contract is already initialized
    fn initialize(e: Env, admin: Address, pegkeeper: Address);

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

    /// Flashloan function for keeping the peg of stablecoins
    ///
    /// ### Arguments
    /// * `pair` - The Address of the AMM pair
    /// * `auction_creator` - The Address of the token
    /// * `liquidation` - The Address of the liquidation contract
    /// * `amount` - The amount of the flashloan
    fn keep_peg(e: Env, name: Symbol, token: Address, amount: i128, args: Vec<Val>);

    /// (Admin only) Set a new address as the admin of this pool
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_admin(e: Env, admin: Address);

    /// (Admin only) Set a new address as the pegkeeper
    ///
    /// ### Arguments
    /// * `pegkeeper` - The new pegkeeper address
    fn set_pegkeeper(e: Env, pegkeeper: Address);
}

#[contractimpl]
impl Treasury for TreasuryContract {

    fn initialize(e: Env, admin: Address, pegkeeper: Address) {
        storage::extend_instance(&e);
        if storage::is_init(&e) {
            panic_with_error!(&e, TreasuryError::AlreadyInitializedError);
        }

        storage::set_pegkeeper(&e, &pegkeeper);
        storage::set_admin(&e, &admin);
    }

    fn add_stablecoin(e: Env, token: Address, blend_pool: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_blend_pool(&e, &token, &blend_pool);
    }

    fn increase_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        // Mint the tokens
        StellarAssetClient::new(&e, &token).mint(&e.current_contract_address(), &amount);

        // Deposit the tokens into the blend pool
        let blend = storage::get_blend_pool(&e, &token);
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
                request_type: 0_u32, // SUPPLY RequestType
                address: token.clone(),
                amount,
            },
        ]);
    }

    fn keep_peg(e: Env, name: Symbol, token: Address, amount: i128, args: Vec<Val>) {
        storage::extend_instance(&e);

        let pegkeeper: Address = storage::get_pegkeeper(&e);
        let blend_pool: Address = storage::get_blend_pool(&e, &token);

        // Mint the tokens to the pegkeeper
        StellarAssetClient::new(&e, &token).mint(&pegkeeper, &amount);

        let token_client = TokenClient::new(&e, &token);

        let func_args: Vec<Val> = vec![
            &e,
            token.into_val(&e),
            amount.into_val(&e),
            args.into_val(&e),
        ];
        e.invoke_contract::<Val>(&pegkeeper, &name, func_args.clone());

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
    }

    fn set_admin(e: Env, new_admin: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        new_admin.require_auth();

        storage::set_admin(&e, &new_admin);
    }

    fn set_pegkeeper(e: Env, new_pegkeeper: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_pegkeeper(&e, &new_pegkeeper);
    }
}
