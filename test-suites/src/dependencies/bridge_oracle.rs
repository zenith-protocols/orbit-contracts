use soroban_sdk::{testutils::Address as _, Address, Env};
mod bridge_oracle_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/bridge_oracle.wasm");
}

pub use bridge_oracle::{BridgeOracleClient, BridgeOracleContract};

pub fn create_bridge_oracle<'a>(e: &Env, contract_id: &Address,  wasm: bool, admin: &Address, oracle: &Address) -> BridgeOracleClient<'a> {
    if wasm {
        e.register_at(&contract_id, bridge_oracle_contract::WASM, (admin, oracle));
    } else {
        e.register_at(&contract_id, BridgeOracleContract {}, (admin, oracle));
    }
    BridgeOracleClient::new(e, &contract_id)
}