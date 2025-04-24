use soroban_sdk::{Address, Env};
mod bridge_oracle_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/bridge_oracle.wasm");
}

pub use bridge_oracle::{BridgeOracleClient, BridgeOracleContract};

pub fn create_bridge_oracle<'a>(e: &Env, contract_id: &Address,  wasm: bool, admin: &Address, stellar_oracle: &Address, other_oracle: &Address) -> BridgeOracleClient<'a> {
    if wasm {
        e.register_at(&contract_id, bridge_oracle_contract::WASM, (admin, stellar_oracle, other_oracle));
    } else {
        e.register_at(&contract_id, BridgeOracleContract {}, (admin, stellar_oracle, other_oracle));
    }
    BridgeOracleClient::new(e, &contract_id)
}