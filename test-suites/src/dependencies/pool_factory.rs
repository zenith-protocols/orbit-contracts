use soroban_sdk::{Address, Env};

mod pool_factory_contract {
    soroban_sdk::contractimport!(file = "../wasm/blend/pool_factory.wasm");
}
pub use pool_factory_contract::{Client as PoolFactoryClient, WASM as POOL_FACTORY_WASM, PoolInitMeta};

pub fn create_pool_factory<'a>(e: &Env, contract_id: &Address, pool_init_meta: PoolInitMeta) -> PoolFactoryClient<'a> {
    e.register_at(&contract_id, POOL_FACTORY_WASM, (pool_init_meta,));
    PoolFactoryClient::new(e, &contract_id)
}
