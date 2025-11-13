use soroban_sdk::{Address, contracttype, Env};
use soroban_sdk::unwrap::UnwrapOptimized;

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days
const LEDGER_THRESHOLD_PERSISTANT: u32 = ONE_DAY_LEDGERS * 100; // ~ 100 days
const LEDGER_BUMP_PERSISTANT: u32 = LEDGER_THRESHOLD_PERSISTANT + 20 * ONE_DAY_LEDGERS; // ~ 120 days

#[derive(Clone)]
#[contracttype]
pub enum TreasuryDataKey {
    ADMIN,
    BLENDPOOL(Address),
    FACTORY,
    TOTALSUPPLY(Address),
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&TreasuryDataKey::ADMIN)
        .unwrap_optimized()
}

pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&TreasuryDataKey::ADMIN, new_admin);
}

pub fn get_factory(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&TreasuryDataKey::FACTORY)
        .unwrap_optimized()
}

pub fn set_factory(e: &Env, new_factory: &Address) {
    e.storage()
        .instance()
        .set(&TreasuryDataKey::FACTORY, new_factory);
}

pub fn get_blend_pool(e: &Env, token_address: &Address) -> Option<Address> {
    let key = TreasuryDataKey::BLENDPOOL(token_address.clone());
    if let Some(result) = e.storage().persistent().get::<TreasuryDataKey, Address>(&key) {
        e.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
        Some(result)
    } else {
        None
    }
}

pub fn set_blend_pool(e: &Env, token_address: &Address, blend_pool: &Address) {
    let key = TreasuryDataKey::BLENDPOOL(token_address.clone());
    e.storage().persistent().set::<TreasuryDataKey, Address>(&key, blend_pool);
    e.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
}

pub fn get_total_supply(e: &Env, reserve_address: &Address) -> i128 {
    let key = TreasuryDataKey::TOTALSUPPLY(reserve_address.clone());
    let total_supply = if let Some(result) = e.storage().persistent().get::<TreasuryDataKey, i128>(&key) {
        e.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
        result
    } else {
        0
    };

    total_supply
}

pub fn set_total_supply(e: &Env, reserve_address: &Address, new_total_supply: &i128) {
    let key = TreasuryDataKey::TOTALSUPPLY(reserve_address.clone());
    e.storage().persistent().set::<TreasuryDataKey, i128>(&key, new_total_supply);
    e.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD_PERSISTANT, LEDGER_BUMP_PERSISTANT);
}