use soroban_sdk::{Address, Env};

mod emitter_contract {
    soroban_sdk::contractimport!(file = "../wasm/blend/emitter.wasm");
}
pub use emitter_contract::{Client as EmitterClient, WASM as EmitterWASM};

pub fn create_emitter<'a>(e: &Env, contract_id: &Address) -> EmitterClient<'a> {
    e.register_at(&contract_id, EmitterWASM, ());
    EmitterClient::new(e, &contract_id)
}
