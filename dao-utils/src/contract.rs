use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::dependencies::treasury::{Client as TreasuryClient};
use crate::dependencies::bridge_oracle::{Client as BridgeOracleClient, Asset};
#[contract]
pub struct DaoUtilsContract;

#[contractimpl]
impl DaoUtilsContract {

    fn new_stablecoin(e: Env, admin: Address, treasury: Address, oracle: Address, token: Address, asset: Asset, blend_pool: Address, initial_supply: i128) {
        admin.require_auth();

        let treasury = storage::get_treasury(&e);
        let treasury_client = TreasuryClient::new(&e, &treasury);
        let bridge_oracle = BridgeOracleClient::new(&e, &oracle);
        let token_asset = Asset::Stellar(token.clone());

        bridge_oracle.add_asset(&token_asset, &asset);
        treasury_client.add_stablecoin(&token, &blend_pool);
        treasury_client.increase_supply(&token, &initial_supply);
    }

    fn update_supply(e: Env, admin: Address, treasury: Address, token: Address, amount: i128) {
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &treasury);
        if amount > 0 {
            treasury.increase_supply(&token, &amount);
        } else {
            treasury.decrease_supply(&token, &amount.abs());
        }
    }
}