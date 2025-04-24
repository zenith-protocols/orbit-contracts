use soroban_sdk::{Address, Env};
use sep_40_oracle::testutils::{MockPriceOracleClient, MockPriceOracleWASM};

pub fn create_mock_oracle<'a>(e: &Env, contract_id: &Address) -> MockPriceOracleClient<'a> {
    e.register_at(&contract_id, MockPriceOracleWASM, ());
    MockPriceOracleClient::new(e, &contract_id)
}
