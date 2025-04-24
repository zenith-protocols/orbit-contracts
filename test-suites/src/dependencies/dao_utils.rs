use soroban_sdk::{testutils::Address as _, Address, Env};

mod dao_utils_wasm {
    use dao_utils::dependencies::bridge_oracle::Asset;
    soroban_sdk::contractimport!(file = "../wasm/orbit/dao_utils.wasm");
}

pub use dao_utils::{DaoUtilsClient, DaoUtilsContract};

pub fn create_dao_utils<'a>(e: &Env, contract_id: &Address, wasm: bool) -> DaoUtilsClient<'a> {
    if wasm {
        e.register_at(&contract_id, dao_utils_wasm::WASM, ());
    } else {
        e.register_at(&contract_id, DaoUtilsContract {}, ());
    }
    DaoUtilsClient::new(e, &contract_id)
}