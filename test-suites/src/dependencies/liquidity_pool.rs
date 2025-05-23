use soroban_sdk::{vec, Address, Env};
use sep_41_token::testutils::MockTokenClient;

mod lp_contract {
    soroban_sdk::contractimport!(file = "../wasm/blend/comet.wasm");
}
pub use lp_contract::{Client as LPClient, WASM as LP_WASM};

/// Deploy a test Comet LP pool of 80% token_1 / 20% token_2. The admin must be the
/// admin of both of the token dependencies used.
///
/// Initializes the pool with the following settings:
/// - Swap fee: 0.3%
/// - Token 1: 1,000
/// - Token 2: 25
/// - Shares: 100
pub(crate) fn create_lp_pool<'a>(
    e: &Env,
    contract_id: &Address,
    admin: &Address,
    token_1: &Address,
    token_2: &Address,
) -> LPClient<'a> {
    e.register_at(&contract_id, LP_WASM, ());
    let client = LPClient::new(e, &contract_id);

    let token_1_client = MockTokenClient::new(e, token_1);
    let token_2_client = MockTokenClient::new(e, token_2);
    token_1_client.mint(&admin, &1_000_0000000);
    token_2_client.mint(&admin, &25_0000000);

    client.init(
        admin,
        &vec![e, token_1.clone(), token_2.clone()],
        &vec![e, 0_8000000, 0_2000000],
        &vec![e, 1_000_0000000, 25_0000000],
        &0_0030000,
    );

    client
}
