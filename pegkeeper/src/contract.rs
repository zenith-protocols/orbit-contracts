use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, Address, Env};
use crate::{errors::PegkeeperError, storage, helper};

#[contract]
pub struct PegkeeperContract;

#[contractclient(name="PegkeeperClient")]
pub trait Pegkeeper {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `admin` - The Address for the admin
    /// * `maximum_duration` - The maximum_duration for swap transaction
    fn initialize(e: Env, admin: Address, router: Address);

    /// Execute operation
    ///
    /// ### Arguments
    /// * `fee_taker` - The Address for the fee taker
    /// * `auction` - The Address for the blend auction
    /// * `token` - The Address for the treasury token
    /// * `collateral_token` - The Address for the collateral token
    /// * `bid_amount` - The amount of the bid
    /// * `lot_amount` - The amount of the lot
    /// * `liq_amount` - The amount of the liquidation
    /// * `blend_pool` - The Address for the blend pool
    /// * `amm` - The Address for the AMM
    fn fl_receive(e: Env, fee_taker: Address, auction: Address, token: Address, collateral_token: Address, bid_amount: i128, lot_amount: i128, liq_amount: i128, blend_pool: Address, amm: Address);
}

#[contractimpl]
impl Pegkeeper for PegkeeperContract {
    fn initialize(e: Env, admin: Address, router: Address) {
        storage::extend_instance(&e);

        if storage::is_init(&e) {
            panic_with_error!(&e, PegkeeperError::AlreadyInitializedError);
        }

        storage::set_router(&e, &router);
        storage::set_admin(&e, &admin);
    }
    fn fl_receive(e: Env, fee_taker: Address, auction: Address, token: Address, collateral_token: Address, bid_amount: i128, lot_amount: i128, liq_amount: i128, blend_pool: Address, amm: Address) {
        storage::extend_instance(&e);

        let admin = storage::get_admin(&e);
        admin.require_auth();

        let token_client = token::Client::new(&e, &token);
        let balance_before = token_client.balance(&e.current_contract_address());

        helper::liquidate(&e, auction, token.clone(), bid_amount.clone(), collateral_token.clone(), lot_amount.clone(), blend_pool.clone(), liq_amount.clone());
        helper::swap(&e, amm, collateral_token.clone(), token.clone(), lot_amount.clone(), 0);

        let balance_after = token_client.balance(&e.current_contract_address());

        if balance_before > balance_after {
            panic_with_error!(&e, PegkeeperError::NotProfitable);
        }

        let profit = balance_after - balance_before;
        token_client.transfer(&e.current_contract_address(), &fee_taker, &profit);

        token_client.approve(
            &e.current_contract_address(),
            &admin,
            &bid_amount,
            &(e.ledger().sequence() + 1),
        );
    }
}
