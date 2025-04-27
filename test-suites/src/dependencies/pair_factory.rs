use soroban_sdk::{Address, Env};

mod pair_factory_contract {
    soroban_sdk::contractimport!(file = "../wasm/soroswap/pair_factory.wasm");
}
pub use pair_factory_contract::{Client as PairFactoryClient, WASM as PAIR_FACTORY_WASM};

pub fn create_pair_factory<'a>(e: &Env, contract_id: &Address) -> PairFactoryClient<'a> {
    e.register_at(&contract_id, PAIR_FACTORY_WASM, ());
    PairFactoryClient::new(e, &contract_id)
}
