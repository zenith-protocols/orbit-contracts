use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, Address, Env, IntoVal};
use crate::errors::MockPegkeeperError;
use crate::{helper, storage};

#[contract]
pub struct MockPegkeeperContract;

#[contractclient(name="MockPegkeeperClient")]
pub trait MockPegkeepeer {

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

    /// Liquidate
    ///
    /// ### Arguments
    /// * `auction` - The Address for the blend auction
    /// * `token` - The Address for the treasury token
    /// * `bid_amount` - The amount of the bid
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the lot
    /// * `blend_pool` - The Address for the blend pool
    /// * `liq_amount` - The amount of the liquidation
    fn liquidate(e: Env, auction: Address, token: Address, bid_amount: i128, collateral_token: Address, lot_amount: i128, blend_pool: Address, liq_amount: i128);

    /// Swap
    ///
    /// ### Arguments
    /// * `amm` - The Address for the AMM
    /// * `token` - The Address for the treasury token
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the lot
    /// * `fee` - The amount of the fee
    fn swap(e: Env, amm: Address, token: Address, collateral_token: Address, lot_amount: i128, fee: i128);

    /// Send profit
    ///
    /// ### Arguments
    /// * `fee_taker` - The Address for the fee taker
    /// * `token` - The Address for the treasury token
    /// * `profit` - The amount of the profit
    fn send_profit(e: Env, fee_taker: Address, token: Address, profit: i128);
}

#[contractimpl]
impl MockPegkeepeer for MockPegkeeperContract {
    fn initialize(e: Env, admin: Address, router: Address) {
        storage::extend_instance(&e);

        if storage::is_init(&e) {
            panic_with_error!(&e, MockPegkeeperError::AlreadyInitializedError);
        }

        storage::set_router(&e, &router);
        storage::set_admin(&e, &admin);
    }
    fn fl_receive(e: Env, fee_taker: Address, auction: Address, token: Address, collateral_token: Address, bid_amount: i128, lot_amount: i128, liq_amount: i128, blend_pool: Address, amm: Address) {
        storage::extend_instance(&e);

        let admin = storage::get_admin(&e);
        admin.require_auth();
    }

    fn liquidate(e: Env, auction: Address, token: Address, bid_amount: i128, collateral_token: Address, lot_amount: i128, blend_pool: Address, liq_amount: i128) {
        storage::extend_instance(&e);

        helper::liquidate(&e, auction, token.clone(), bid_amount.clone(), collateral_token.clone(), lot_amount.clone(), blend_pool.clone(), liq_amount.clone());
    }

    fn swap(e: Env, amm: Address, token: Address, collateral_token: Address, lot_amount: i128, fee: i128) {
        storage::extend_instance(&e);

        helper::swap(&e, amm, collateral_token.clone(), token.clone(), lot_amount.clone(), fee.clone());
    }

    fn send_profit(e: Env, fee_taker: Address, token: Address, profit: i128) {
        storage::extend_instance(&e);

        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&e.current_contract_address(), &fee_taker, &profit);
    }
}
