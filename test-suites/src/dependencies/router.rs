use soroban_sdk::{testutils::Address as _, Address, Env};

mod router_contract {
    soroban_sdk::contractimport!(file = "../wasm/soroswap/router.wasm");
}
pub use router_contract::{Client as RouterClient, WASM as ROUTER_WASM};

pub fn create_router<'a>(e: &Env, contract_id: &Address) -> RouterClient<'a> {
    e.register_at(&contract_id, ROUTER_WASM, ());
    RouterClient::new(e, &contract_id)
}
