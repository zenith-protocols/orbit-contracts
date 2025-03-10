pub use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{contract, contractclient, contractimpl, vec, Address, Env, Symbol, Vec, Val, IntoVal, BytesN};
use crate::storage;

#[contract]
pub struct BridgeOracleContract;

#[contractclient(name = "BridgeOracleClient")]
pub trait BridgeOracle {

    /// (Admin only) Add a new asset to the oracle
    /// # Arguments
    /// * `asset` - The asset to add
    /// * `to` - The asset to convert to
    fn add_asset(e: Env, asset: Asset, to: Asset);

    /// (Admin only) Set a new oracle for the bridge oracle
    /// # Arguments
    /// * `oracle` - The new oracle address
    fn set_oracle(e: Env, oracle: Address);

    /// Fetch the number of decimals for the oracle
    fn decimals(env: Env) -> u32;

    /// Fetch the last price for the asset
    /// # Arguments
    /// * `asset` - The asset to fetch the price for
    fn lastprice(env: Env, asset: Asset) -> Option<PriceData>;

    /// Updates this contract to a new version
    /// # Arguments
    /// * `new_wasm_hash` - The new wasm hash
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
}

#[contractimpl]
impl BridgeOracleContract {

    /// Initializes the bridge oracle
    /// # Arguments
    /// * `admin` - The admin address
    /// * `oracle` - The oracle contract address
    pub fn __constructor(e: Env, admin: Address, oracle: Address) {
        admin.require_auth();

        storage::set_admin(&e, &admin);
        storage::set_oracle(&e, &oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "init")), (admin.clone(), oracle.clone()));
    }
}

#[contractimpl]
impl BridgeOracle for BridgeOracleContract {

    fn add_asset(e: Env, asset: Asset, to: Asset) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_bridge_asset(&e, &asset, &to);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "add_asset")), (asset.clone(), to.clone()));
    }

    fn set_oracle(e: Env, oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_oracle(&e, &oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "set_oracle")), oracle.clone());
    }

    fn decimals(env: Env) -> u32 {
        storage::extend_instance(&env);
        let oracle = storage::get_oracle(&env);
        env.invoke_contract::<u32>(&oracle, &Symbol::new(&env, "decimals"), vec![&env])
    }

    fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&env);
        let to_asset = storage::get_bridge_asset(&env, &asset);
        let oracle = storage::get_oracle(&env);

        let args: Vec<Val> = vec![&env,
                                      to_asset.into_val(&env)];
        env.invoke_contract::<Option<PriceData>>(&oracle, &Symbol::new(&env, "lastprice"), args)
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}