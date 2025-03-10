use soroban_sdk::{Address, Env, contracttype};
use sep_40_oracle::Asset;
use soroban_sdk::unwrap::UnwrapOptimized;

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days
const LEDGER_THRESHOLD_PERSISTANT: u32 = ONE_DAY_LEDGERS * 100; // ~ 100 days
const LEDGER_BUMP_PERSISTANT: u32 = LEDGER_THRESHOLD_PERSISTANT + 20 * ONE_DAY_LEDGERS; // ~ 120 days

#[derive(Clone)]
#[contracttype]
pub enum BridgeOracleDataKey {
    ADMIN,
    ORACLE,
    BRIDGE(Asset),
}

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&BridgeOracleDataKey::ADMIN)
        .unwrap_optimized()
}

pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&BridgeOracleDataKey::ADMIN, new_admin);
}

pub fn get_bridge_asset(env: &Env, asset: &Asset) -> Asset {
    let key = BridgeOracleDataKey::BRIDGE(asset.clone());
    if let Some(result) = env.storage().persistent().get::<BridgeOracleDataKey, Asset>(&key) {
        env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
        result
    } else {
        asset.clone()
    }
}

pub fn set_bridge_asset(env: &Env, asset: &Asset, to: &Asset) {
    let key = BridgeOracleDataKey::BRIDGE(asset.clone());
    env.storage().persistent().set::<BridgeOracleDataKey, Asset>(&key, to);
    env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
}

pub fn get_oracle(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&BridgeOracleDataKey::ORACLE)
        .unwrap_optimized()
}

pub fn set_oracle(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&BridgeOracleDataKey::ORACLE, address);
}