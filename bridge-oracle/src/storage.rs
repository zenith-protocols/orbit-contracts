use soroban_sdk::{Address, Env, contracttype, IntoVal, Val, TryFromVal};
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

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>, F: FnOnce() -> V>(
    e: &Env,
    key: &K,
    default: F,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default()
    }
}

pub fn is_init(e: &Env) -> bool { e.storage().instance().has(&BridgeOracleDataKey::ADMIN) }

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
    get_persistent_default(
        env,
        &key,
        || asset.clone(),
        LEDGER_THRESHOLD_PERSISTANT,
        LEDGER_BUMP_PERSISTANT,
    )
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