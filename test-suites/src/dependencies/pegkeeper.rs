use soroban_sdk::{testutils::Address as _, Address, Env};

mod pegkeeper_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/pegkeeper.wasm");
}

pub use pegkeeper::{PegkeeperClient, PegkeeperContract};

pub fn create_pegkeeper<'a>(e: &Env, contract_id: &Address, wasm: bool, treasury: &Address, router: &Address) -> PegkeeperClient<'a> {
    if wasm {
        e.register_at(&contract_id, pegkeeper_contract::WASM, (treasury, router));
    } else {
        e.register_at(&contract_id, PegkeeperContract {}, (treasury, router));
    }
    PegkeeperClient::new(e, &contract_id)
}
