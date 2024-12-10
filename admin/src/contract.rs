use soroban_sdk::{contract, contractclient, contractimpl, Address, Env, Vec, IntoVal, panic_with_error, token};
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
    /// * `treasury` - The treasury address
    /// * `bridge_oracle` - The bridge oracle address
    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address);

    /// Creates a new stablecoin
    /// # Arguments
    /// * `token` - The address of the token to add
    /// * `asset` - The asset of the fiat currency to peg the new token to
    /// * `blend_pool` - The address of the blend pool
    /// * `initial_supply` - The initial supply of the new token lended to the blend pool
    fn new_stablecoin(e: Env, token: Address, asset: Asset, blend_pool: Address, initial_supply: i128);

    /// Updates the pegkeeper
    /// # Arguments
    /// * `pegkeeper` - The address of the pegkeeper
    fn update_pegkeeper(e: Env, pegkeeper: Address);

    /// Updates the oracle
    /// # Arguments
    /// * `oracle` - The address of the oracle
    fn update_oracle(e: Env, oracle: Address);

    /// Updates the supply of a token
    /// # Arguments
    /// * `token` - The address of the token
    /// * `amount` - The amount to update the supply by can be positive or negative
    fn update_supply(e: Env, token: Address, amount: i128);

    /// Updates the blend pool
    /// # Arguments
    /// * `pool` - The address of the blend pool
    /// * `backstop_take_rate` - The backstop take rate
    /// * `max_positions` - The maximum number of positions
    fn update_pool(e: Env, pool: Address, backstop_take_rate: u32, max_positions: u32);

    /// Sets the reserve of a blend pool
    /// # Arguments
    /// * `pool` - The address of the blend pool
    /// * `asset` - The address of the asset
    /// * `metadata` - The reserve configuration
    fn set_reserve(e: Env, pool: Address, asset: Address, metadata: ReserveConfig) -> u32;

    /// Sets the emissions configuration of a blend pool
    /// # Arguments
    /// * `pool` - The address of the blend pool
    /// * `res_emission_metadata` - The reserve emission metadata
    fn set_emissions_config(e: Env, pool: Address, res_emission_metadata: Vec<ReserveEmissionMetadata>);

    /// Sets the status of a blend pool
    /// # Arguments
    /// * `pool` - The address of the blend pool
    /// * `pool_status` - The status of the blend pool
    fn set_status(e: Env, pool: Address, pool_status: u32);

    /// Sets the admin address
    /// # Arguments
    /// * `admin` - The address of the new admin
    fn set_admin(e: Env, admin: Address);

}

#[contractimpl]
impl Admin for AdminContract {

    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address) {
        storage::extend_instance(&e);
        if storage::is_init(&e) {
            panic_with_error!(&e, AdminError::AlreadyInitializedError);
        }
        storage::set_admin(&e, &admin);
        storage::set_treasury(&e, &treasury);
        storage::set_bridge_oracle(&e, &bridge_oracle);
    }

    fn new_stablecoin(e: Env, token: Address, asset: Asset, blend_pool: Address, initial_supply: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let treasury = storage::get_treasury(&e);
        let treasury_client = TreasuryClient::new(&e, &treasury);
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        let token_asset = Asset::Stellar(token.clone());
        let token_client = token::StellarAssetClient::new(&e, &token);

        bridge_oracle.add_asset(&token_asset, &asset);
        treasury_client.add_stablecoin(&token, &blend_pool);
        token_client.set_admin(&treasury);
        treasury_client.increase_supply(&token, &initial_supply);
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

    fn set_reserve(e: Env, pool: Address, asset: Address, metadata: ReserveConfig) -> u32 {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let pool_client = PoolClient::new(&e, &pool);
        pool_client.queue_set_reserve(&asset, &metadata);
        pool_client.set_reserve(&asset)
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

    fn set_admin(e: Env, admin: Address) {
        storage::extend_instance(&e);
        let current_admin = storage::get_admin(&e);
        current_admin.require_auth();
        storage::set_admin(&e, &admin);
    }
}