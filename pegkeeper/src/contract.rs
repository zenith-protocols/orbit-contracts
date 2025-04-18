use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, Address, Env, Symbol};
use crate::{errors::PegkeeperError, storage, helper};

#[contract]
pub struct PegkeeperContract;

#[contractclient(name="PegkeeperClient")]
pub trait Pegkeeper {

    /// Execute operation
    ///
    /// ### Arguments
    /// * `token` - The Address for the token
    /// * `amount` - The amount received of the token
    /// * `blend_pool` - The Address for the blend pool
    /// * `auction` - The Address for the auction
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the collateral token to withdraw after liquidation
    /// * `liq_amount` - The amount to liquidate in percentage 0-100
    /// * `amm` - The Address for the AMM
    /// * `min_profit` - The minimum profit acceptable
    /// * `fee_taker` - The Address for the fee taker
    fn fl_receive(e: Env, token: Address, amount: i128, blend_pool: Address, auction: Address, collateral_token: Address, lot_amount: i128, liq_amount: i128, amm: Address, min_profit: i128, fee_taker: Address);
}

#[contractimpl]
impl PegkeeperContract {

    /// Initializes the PegKeeper contract
    ///
    /// ### Arguments
    /// * `treasury` - The Address of the treasury
    /// * `router` - The address of the soroswap router
    pub fn __constructor(e: Env, treasury: Address, router: Address) {
        storage::set_treasury(&e, &treasury);
        storage::set_router(&e, &router);
        e.events().publish(("Pegkeeper", Symbol::new(&e, "init")), (treasury.clone(), router.clone()));
    }
}

#[contractimpl]
impl Pegkeeper for PegkeeperContract {

    fn fl_receive(e: Env, token: Address, amount: i128, blend_pool: Address, auction: Address, collateral_token: Address, lot_amount: i128, liq_amount: i128, amm: Address, min_profit: i128, fee_taker: Address) {
        storage::extend_instance(&e);

        let treasury = storage::get_treasury(&e);
        treasury.require_auth();

        let token_client = token::TokenClient::new(&e, &token);
        let collateral_client = token::TokenClient::new(&e, &collateral_token);
        let balance_before = token_client.balance(&e.current_contract_address());
        let collateral_balance = collateral_client.balance(&e.current_contract_address());

        let positions = helper::liquidate(&e, auction, token.clone(), amount.clone(), collateral_token.clone(), lot_amount.clone(), blend_pool.clone(), liq_amount.clone());
        if positions.liabilities.len() > 0 || positions.collateral.len() > 0 {
            panic_with_error!(&e, PegkeeperError::PositionStillOpen);
        }

        let collateral_balance_after = collateral_client.balance(&e.current_contract_address());
        let to_swap = collateral_balance_after - collateral_balance;

        helper::swap(&e, amm, collateral_token.clone(), token.clone(), to_swap.clone(), 0);

        let balance_after = token_client.balance(&e.current_contract_address());

        let profit = balance_after - balance_before;
        if profit < min_profit {
            panic_with_error!(&e, PegkeeperError::NotProfitable);
        }
        token_client.transfer(&e.current_contract_address(), &fee_taker, &profit);

        token_client.approve(
            &e.current_contract_address(),
            &treasury,
            &amount,
            &(e.ledger().sequence() + 1),
        );

        e.events().publish(("Pegkeeper", Symbol::new(&e, "fl_receive")), (token.clone(), amount.clone()));
    }
}
