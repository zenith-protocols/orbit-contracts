#[cfg(test)]

use sep_40_oracle::Asset;
use sep_40_oracle::testutils::{MockPriceOracleClient, MockPriceOracleWASM};
use sep_40_oracle::testutils::Asset as TestAsset;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol, vec as svec, symbol_short};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use crate::{BridgeOracleContract, BridgeOracleClient};

pub(crate) fn create_mock_oracle(e: &Env) -> (Address, MockPriceOracleClient) {
    let contract_address = e.register(MockPriceOracleWASM, ());
    (
        contract_address.clone(),
        MockPriceOracleClient::new(e, &contract_address),
    )
}

#[test]
fn test_add_assets() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    // We don't need actual token contracts for this test
    let token1 = Address::generate(&env);
    let token2 = Address::generate(&env);
    let (oracle_address, mock_oracle_client) = create_mock_oracle(&env);

    mock_oracle_client.set_data(
        &admin.clone(),
        &TestAsset::Other(Symbol::new(&env, "USD")),
        &svec![
                &env,
                TestAsset::Other(Symbol::new(&env, "USD")),
                TestAsset::Other(Symbol::new(&env, "EURO")),
            ],
        &7,
        &300,
    );
    mock_oracle_client.set_price_stable(&svec![
            &env,
            1_0000000,
            1_1000000,
        ]);

    let bridge_oracle_address = env.register(BridgeOracleContract, (admin.clone(), oracle_address));
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);

    let stellar_asset = Asset::Stellar(token1.clone());
    let stellar_asset2 = Asset::Stellar(token2.clone());
    let usd_asset = Asset::Other(Symbol::new(&env, "USD"));
    let euro_asset = Asset::Other(Symbol::new(&env, "EURO"));
    bridge_oracle_client.add_asset(
        &stellar_asset,
        &usd_asset,
    );
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    bridge_oracle_address.clone(),
                    symbol_short!("add_asset"),
                    (stellar_asset.clone(), usd_asset.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    bridge_oracle_client.add_asset(
        &stellar_asset2,
        &euro_asset,
    );

    let token1_price = bridge_oracle_client.lastprice(&Asset::Stellar(token1.clone())).unwrap().price;
    let token2_price = bridge_oracle_client.lastprice(&Asset::Stellar(token2.clone())).unwrap().price;
    let usd_price = bridge_oracle_client.lastprice(&Asset::Other(Symbol::new(&env, "USD"))).unwrap().price;
    let euro_price = bridge_oracle_client.lastprice(&Asset::Other(Symbol::new(&env, "EURO"))).unwrap().price;

    assert_eq!(usd_price, 1_0000000);
    assert_eq!(euro_price, 1_1000000);
    assert_eq!(token1_price, usd_price);
    assert_eq!(token2_price, euro_price);
}

#[test]
fn test_update_oracle() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let stellar_oracle = Address::generate(&env);
    let other_oracle = Address::generate(&env);

    let bridge_oracle_address = env.register(BridgeOracleContract, (admin.clone(), stellar_oracle, other_oracle));
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);


    let new_stellar_oracle = Address::generate(&env);
    let new_other_oracle = Address::generate(&env);

    bridge_oracle_client.set_stellar_oracle(&new_stellar_oracle);
    bridge_oracle_client.set_other_oracle(&new_other_oracle);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    bridge_oracle_address.clone(),
                    Symbol::new(&env, "set_stellar_oracle"),
                    (new_stellar_oracle.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    bridge_oracle_address.clone(),
                    Symbol::new(&env, "set_stellar_oracle"),
                    (new_other_oracle.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}