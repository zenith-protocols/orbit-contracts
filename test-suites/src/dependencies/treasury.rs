use soroban_sdk::{Address, Env};

mod treasury_contract {
    soroban_sdk::contractimport!(
        file = "../wasm/orbit/treasury.wasm"
    );
}

pub use treasury_contract::WASM as POOL_WASM;
pub use treasury::{TreasuryClient, TreasuryContract};

pub fn create_treasury<'a>(e: &Env, contract_id: &Address, wasm: bool, admin: &Address, factory: &Address, pegkeeper: &Address) -> TreasuryClient<'a> {
    if wasm {
        e.register_at(&contract_id, treasury_contract::WASM, (admin, factory, pegkeeper));
    } else {
        e.register_at(&contract_id, TreasuryContract {}, (admin, factory, pegkeeper));
    }
    TreasuryClient::new(e, &contract_id)
}