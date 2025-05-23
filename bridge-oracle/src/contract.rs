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
    /// * `stellar_oracle` - The stellar oracle address
    /// * `other_oracle` - The other oracle address
    fn set_oracles(e: Env, stellar_oracle: Address, other_oracle: Address);

    /// Fetch the number of decimals for the stellar oracle
    fn decimals(env: Env) -> u32;

    /// Fetch the last price for the asset
    /// # Arguments
    /// * `asset` - The asset to fetch the price for
    fn lastprice(env: Env, asset: Asset) -> Option<PriceData>;

    /// (Admin only) Set the admin address
    /// # Arguments
    /// * `new_admin` - The new admin address
    fn set_admin(e: Env, new_admin: Address);

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

    fn set_oracles(e: Env, stellar_oracle: Address, other_oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_stellar_oracle(&e, &stellar_oracle);
        storage::set_other_oracle(&e, &other_oracle);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "set_oracles")), (stellar_oracle.clone(), other_oracle.clone()));
    }

    fn decimals(env: Env) -> u32 {
        storage::extend_instance(&env);
        let oracle = storage::get_stellar_oracle(&env);
        env.invoke_contract::<u32>(&oracle, &Symbol::new(&env, "decimals"), vec![&env])
    }

    fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&env);
        let to_asset = storage::get_bridge_asset(&env, &asset);

        match to_asset.clone() {
            Asset::Stellar(_) => {
                let stellar_oracle = storage::get_stellar_oracle(&env);
                let args: Vec<Val> = vec![&env, to_asset.into_val(&env)];
                env.invoke_contract::<Option<PriceData>>(&stellar_oracle, &Symbol::new(&env, "lastprice"), args)
            }
            Asset::Other(name) => {
                if name == Symbol::new(&env, "USD") {
                    let timestamp = env.ledger().timestamp();
                    Some(PriceData {price: 1_00_000_000_000_000, timestamp})
                }
                else {
                    let other_oracle = storage::get_other_oracle(&env);
                    let args: Vec<Val> = vec![&env, to_asset.into_val(&env)];
                    env.invoke_contract::<Option<PriceData>>(&other_oracle, &Symbol::new(&env, "lastprice"), args)
                }   
            }
        }
    }

    fn set_admin(e: Env, new_admin: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_admin(&e, &new_admin);

        e.events().publish(("BridgeOracle", Symbol::new(&e, "set_admin")), (new_admin.clone(),));
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}