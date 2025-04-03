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

    /// (Admin only) Set a new stellar oracle for the bridge oracle
    /// # Arguments
    /// * `oracle` - The new stellar oracle address
    fn set_stellar_oracle(e: Env, oracle: Address);

    /// (Admin only) Set a new other oracle for the bridge oracle
    /// # Arguments
    /// * `oracle` - The new other oracle address
    fn set_other_oracle(e: Env, oracle: Address);

    /// Fetch the number of decimals for the stellar oracle
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
    /// * `stellar_oracle` - The oracle contract address for stellar asset
    /// * `other_oracle` - The oracle contract address for other asset
    pub fn __constructor(e: Env, admin: Address, stellar_oracle: Address, other_oracle: Address) {
        admin.require_auth();

        storage::set_admin(&e, &admin);
        storage::set_stellar_oracle(&e, &stellar_oracle);
        storage::set_other_oracle(&e, &other_oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "init")), (admin.clone(), stellar_oracle.clone(), other_oracle.clone()));
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

    fn set_stellar_oracle(e: Env, oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_stellar_oracle(&e, &oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "set_stellar_oracle")), oracle.clone());
    }

    fn set_other_oracle(e: Env, oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_other_oracle(&e, &oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "set_other_oracle")), oracle.clone());
    }

    fn decimals(env: Env) -> u32 {
        storage::extend_instance(&env);
        let oracle = storage::get_stellar_oracle(&env);
        env.invoke_contract::<u32>(&oracle, &Symbol::new(&env, "decimals"), vec![&env])
    }

    fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&env);
        let to_asset = storage::get_bridge_asset(&env, &asset);

        let stellar_oracle = storage::get_stellar_oracle(&env);
        let other_oracle = storage::get_other_oracle(&env);

        match to_asset.clone() {
            Asset::Stellar(a) => {
                let args: Vec<Val> = vec![&env, to_asset.into_val(&env)];
                env.invoke_contract::<Option<PriceData>>(&stellar_oracle, &Symbol::new(&env, "lastprice"), args)
            }
            Asset::Other(a) => {
                if a == Symbol::new(&env, "USD") {
                    let timestamp = env.ledger().timestamp();
                    Some(PriceData {price: 1_00_000_000_000_000, timestamp})
                }
                else {
                    let args: Vec<Val> = vec![&env, to_asset.into_val(&env)];

                    env.invoke_contract::<Option<PriceData>>(&other_oracle, &Symbol::new(&env, "lastprice"), args)
                }   
            }
        }
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}