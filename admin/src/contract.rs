use soroban_sdk::{contract, contractclient, contractimpl, vec, Address, Env, Symbol, Vec, Val, IntoVal, panic_with_error};
use crate::error::AdminError;
use crate::storage;
use crate::dependencies::pool::{Client as PoolClient, ReserveConfig, ReserveEmissionMetadata};
use crate::dependencies::treasury::{Client as TreasuryClient};
use crate::dependencies::bridge_oracle::{Client as BridgeOracleClient, Asset};
#[contract]
pub struct AdminContract;

#[contractclient(name = "AdminClient")]
pub trait Admin {

    /// Initializes the bridge oracle
    /// # Arguments
    /// * `admin` - The admin address
    /// * `oracle` - The oracle contract address
    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address);

    fn new_stablecoin(e: Env, token_a: Address, token_b: Address, blend_pool: Address, initial_supply: i128);

    fn update_pegkeeper(e: Env, pegkeeper: Address);

    fn update_oracle(e: Env, oracle: Address);

    fn update_supply(e: Env, token: Address, amount: i128);

    fn update_pool(e: Env, pool: Address, backstop_take_rate: u32, max_positions: u32);

    fn set_reserve(e: Env, pool: Address, asset: Address, metadata: ReserveConfig);

    fn set_emissions_config(e: Env, pool: Address, res_emission_metadata: Vec<ReserveEmissionMetadata>);

    fn set_status(e: Env, pool: Address, pool_status: u32);

}

#[contractimpl]
impl Admin for AdminContract {

    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address) {
        storage::extend_instance(&e);
        if storage::is_init(&e) {
            panic_with_error!(AdminError::AlreadyInitializedError);
        }
        storage::set_admin(&e, &admin);
        storage::set_treasury(&e, &treasury);
        storage::set_bridge_oracle(&e, &bridge_oracle);
    }

    fn new_stablecoin(e: Env, token_a: Address, asset: Asset, blend_pool: Address, initial_supply: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        let token_asset = Asset { //TODO: Check this
            Stellar: token_a.clone()
        };
        bridge_oracle.add_asset(&token_asset, &asset);
        treasury.add_stablecoin(&token_a, &blend_pool);
        treasury.increase_supply(&token_a, &initial_supply);
    }

    fn update_pegkeeper(e: Env, pegkeeper: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        treasury.set_pegkeeper(&pegkeeper);
    }

    fn update_oracle(e: Env, oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        bridge_oracle.set_oracle(&oracle);
    }

    fn update_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        if amount > 0 {
            treasury.increase_supply(&token, &amount);
        } else {
            treasury.decrease_supply(&token, &amount.abs());
        }
    }

    fn update_pool(e: Env, pool: Address, backstop_take_rate: u32, max_positions: u32) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        PoolClient::new(&e, &pool).update_pool(&backstop_take_rate, &max_positions);
    }

    fn set_reserve(e: Env, pool: Address, asset: Address, metadata: ReserveConfig) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let pool_client = PoolClient::new(&e, &pool);
        pool_client.queue_set_reserve(&asset, &metadata);
        pool_client.set_reserve(&asset);
    }

    fn set_emissions_config(e: Env, pool: Address, res_emission_metadata: Vec<ReserveEmissionMetadata>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        PoolClient::new(&e, &pool).set_emissions_config(&res_emission_metadata);
    }

    fn set_status(e: Env, pool: Address, pool_status: u32) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        PoolClient::new(&e, &pool).set_status(&pool_status);
    }
}