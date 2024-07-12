use soroban_sdk::{Address, Env, contracttype};
use sep_40_oracle::Asset;
use soroban_sdk::unwrap::UnwrapOptimized;

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger

const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days

#[derive(Clone)]
#[contracttype]
pub enum BridgeOracleDataKey {
    ADMIN,
    ORACLE,
    BRIDGE(Asset),
}

/// Bump the instance rent for the contract
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

/// Check if the contract has been initialized
pub fn is_init(e: &Env) -> bool { e.storage().instance().has(&BridgeOracleDataKey::ADMIN) }

/// Fetch the current admin Address
///
/// ### Panics
/// If the admin does not exist
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&BridgeOracleDataKey::ADMIN)
        .unwrap_optimized()
}

/// Set a new admin
///
/// ### Arguments
/// * `new_admin` - The Address for the admin
pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&BridgeOracleDataKey::ADMIN, new_admin);
}

/// Fetch the asset to convert to
///
/// ### Arguments
/// * `asset` - The asset to convert from
pub fn get_bridge_asset(env: &Env, asset: &Asset) -> Asset {
    env.storage()
        .instance()
        .get(&BridgeOracleDataKey::BRIDGE(asset.clone()))
        .unwrap_or_else(|| asset.clone())
}

/// Set the asset to convert to
///
/// ### Arguments
/// * `asset` - The asset to convert from
/// * `to` - The asset to convert to
pub fn set_bridge_asset(env: &Env, asset: &Asset, to: &Asset) {
    env.storage()
        .instance()
        .set(&BridgeOracleDataKey::BRIDGE(asset.clone()), to);
}

/// Fetch the current oracle Address
///
/// ### Panics
/// If the oracle does not exist
pub fn get_oracle(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&BridgeOracleDataKey::ORACLE)
        .unwrap_optimized()
}

/// Set a new oracle
///
/// ### Arguments
/// * `address` - The Address for the oracle
pub fn set_oracle(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&BridgeOracleDataKey::ORACLE, address);
}