use soroban_sdk::{Address, Env, vec, Val, Vec, IntoVal, Symbol};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};

use crate::dependencies::{
    router::{Client as RouterClient},
    pool::{Client as PoolClient, Request},
};
use crate::dependencies::pool::Positions;
use crate::storage;

const REQUEST_TYPE_LIQUIDATION_AUCTION: u32 = 6;
const REQUEST_TYPE_REPAY: u32 = 5;
const REQUEST_TYPE_COLLATERAL_WITHDRAWAL: u32 = 3;

pub fn liquidate(e: &Env, auction_creator: Address, token_a: Address, token_a_bid_amount: i128, token_b: Address, token_b_lot_amount: i128, blend_pool: Address, liq_amount: i128) -> Positions {
  let fill_requests = vec![
      e,
      Request {
          request_type: REQUEST_TYPE_LIQUIDATION_AUCTION,
          address: auction_creator.clone(),
          amount: liq_amount.clone(),
      },
      Request {
          request_type: REQUEST_TYPE_REPAY,
          address: token_a.clone(),
          amount: token_a_bid_amount,
      },
      Request {
          request_type: REQUEST_TYPE_COLLATERAL_WITHDRAWAL,
          address: token_b.clone(),
          amount: token_b_lot_amount,
      },
  ];

  let args: Vec<Val> = vec![
      e,
      e.current_contract_address().into_val(e),
      blend_pool.into_val(e),
      token_a_bid_amount.into_val(e),
  ];
  e.authorize_as_current_contract(vec![
      e,
      InvokerContractAuthEntry::Contract(SubContractInvocation {
          context: ContractContext {
              contract: token_a.clone(),
              fn_name: Symbol::new(e, "transfer"),
              args: args.clone(),
          },
          sub_invocations: vec![e],
      })
  ]);

  PoolClient::new(e, &blend_pool).submit(&e.current_contract_address(), &e.current_contract_address(), &e.current_contract_address(), &fill_requests)
}

pub fn swap(e: &Env, pair: Address, token_a: Address, token_b: Address, amount_a: i128, amount_b: i128) {
  let router = storage::get_router(e);
  let router_client = RouterClient::new(e, &router);

  let path = vec![
      e,
      token_a.clone(),
      token_b.clone(),
  ];
  let args: Vec<Val> = vec![
      e,
      e.current_contract_address().into_val(e),
      pair.into_val(e),
      amount_a.into_val(e),
  ];
  e.authorize_as_current_contract(vec![
      e,
      InvokerContractAuthEntry::Contract( SubContractInvocation {
          context: ContractContext {
              contract: token_a.clone(),
              fn_name: Symbol::new(e, "transfer"),
              args: args.clone(),
          },
          sub_invocations: vec![e]
      })
  ]);
  let _ = router_client.swap_exact_tokens_for_tokens(&amount_a, &amount_b, &path, &e.current_contract_address(), &u64::MAX);
}