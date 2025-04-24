#[cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use crate::{TreasuryContract, TreasuryClient};

#[test]
fn test_update_pegkeeper() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = Address::generate(&env);
    let pegkeeper = Address::generate(&env);

    let treasury_address = env.register(TreasuryContract, (admin.clone(), factory, pegkeeper));
    let treasury_client = TreasuryClient::new(&env, &treasury_address);

    let new_pegkeeper = Address::generate(&env);

    treasury_client.set_pegkeeper(&new_pegkeeper);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    treasury_address.clone(),
                    Symbol::new(&env, "set_pegkeeper"),
                    (new_pegkeeper.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}