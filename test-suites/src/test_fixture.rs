use std::collections::HashMap;
use std::ops::Index;

use crate::dependencies::backstop::create_backstop;
use crate::dependencies::emitter::create_emitter;
use crate::dependencies::liquidity_pool::{create_lp_pool, LPClient};
use crate::dependencies::oracle::create_mock_oracle;
use crate::dependencies::pool::POOL_WASM;
use crate::dependencies::pool_factory::create_pool_factory;
use crate::dependencies::token::{create_stellar_token};
use crate::dependencies::backstop::BackstopClient;
use crate::dependencies::emitter::EmitterClient;
use crate::dependencies::pool::{
    PoolClient, PoolConfig, PoolDataKey, ReserveConfig, ReserveData,
};
use crate::dependencies::pool_factory::{PoolFactoryClient, PoolInitMeta};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient};
use sep_41_token::testutils::MockTokenClient;
use soroban_sdk::testutils::{Address as _, BytesN as _, Ledger, LedgerInfo};
use soroban_sdk::{vec as svec, Address, BytesN, Env, String, Map, Symbol};

use crate::dependencies::pair::{PAIR_WASM, PairClient};
use crate::dependencies::pair_factory::{PairFactoryClient, create_pair_factory};
use crate::dependencies::router::{RouterClient, create_router};
use crate::dependencies::dao_utils::{DaoUtilsClient, create_dao_utils};
use crate::dependencies::bridge_oracle::{BridgeOracleClient, create_bridge_oracle};
use crate::dependencies::treasury::{TreasuryClient, create_treasury};
use crate::dependencies::pegkeeper::{PegkeeperClient, create_pegkeeper};

pub const SCALAR_7: i128 = 1_000_0000;
pub const SCALAR_9: i128 = 1_000_000_000;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TokenIndex {
    BLND = 0,
    USDC = 1,
    XLM = 2,
    OUSD = 3,
    OEURO = 4,
    OGBP = 5,
}

pub struct PoolFixture<'a> {
    pub pool: PoolClient<'a>,
    pub reserves: HashMap<TokenIndex, u32>,
}

impl<'a> Index<TokenIndex> for Vec<MockTokenClient<'a>> {
    type Output = MockTokenClient<'a>;

    fn index(&self, index: TokenIndex) -> &Self::Output {
        &self[index as usize]
    }
}

pub struct TestFixture<'a> {
    pub env: Env,
    pub admin: Address,
    pub users: Vec<Address>,
    pub emitter: EmitterClient<'a>,
    pub backstop: BackstopClient<'a>,
    pub pool_factory: PoolFactoryClient<'a>,
    pub oracle: MockPriceOracleClient<'a>,
    pub lp: LPClient<'a>,
    pub pools: Vec<PoolFixture<'a>>,
    pub tokens: Vec<MockTokenClient<'a>>,
    pub pair_factory: PairFactoryClient<'a>,
    pub pairs: Vec<PairClient<'a>>,
    pub router: RouterClient<'a>,
    pub dao_utils: DaoUtilsClient<'a>,
    pub bridge_oracle: BridgeOracleClient<'a>,
    pub treasury: TreasuryClient<'a>,
    pub pegkeeper: PegkeeperClient<'a>,
}

impl TestFixture<'_> {
    /// Create a new TestFixture for the Orbit Protocol
    ///
    /// Deploys BLND (0), USDC (1), wETH (2), XLM (3), and STABLE (4) test tokens, alongside all required
    /// Blend Protocol dependencies, including a BLND-USDC LP.
    
    pub fn create<'a>(wasm: bool) -> TestFixture<'a> {
        let e = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let admin = Address::generate(&e);
        let frodo = Address::generate(&e);

        e.ledger().set(LedgerInfo {
            timestamp: 1441065600, // Sept 1st, 2015 (backstop epoch)
            protocol_version: 22,
            sequence_number: 150,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 500000,
            min_persistent_entry_ttl: 500000,
            max_entry_ttl: 9999999,
        });

        let (blnd_id, blnd_client) = create_stellar_token(&e, &admin);
        let (usdc_id, usdc_client) = create_stellar_token(&e, &admin);
        let (xlm_id, xlm_client) = create_stellar_token(&e, &admin);
        let (_, ousd_client) = create_stellar_token(&e, &admin);

        // create addresses for all contracts
        let backstop_id = Address::generate(&e);
        let emitter_id = Address::generate(&e);
        let pool_factory_id = Address::generate(&e);
        let mock_oracle_id = Address::generate(&e);
        let lp_id = Address::generate(&e);
        let pair_factory_id = Address::generate(&e);
        let router_id = Address::generate(&e);
        let treasury_id = Address::generate(&e);
        let bridge_oracle_id = Address::generate(&e);
        let pegkeeper_id = Address::generate(&e);
        let dao_utils_id = Address::generate(&e);

        // deploy Blend Protocol dependencies
        let lp_client = create_lp_pool(&e, &lp_id, &admin, &blnd_id, &usdc_id);
        let emitter_client = create_emitter(&e, &emitter_id);

        blnd_client.set_admin(&emitter_id);
        emitter_client.initialize(&blnd_id, &backstop_id, &lp_id);

        // initialize backstop
        let backstop_client = create_backstop(
            &e,
            &backstop_id,
            &lp_id,
            &emitter_id,
            &blnd_id,
            &usdc_id,
            &pool_factory_id,
            &svec![
                &e,
                (admin.clone(), 10_000_000 * SCALAR_7),
                (frodo.clone(), 30_000_000 * SCALAR_7)
            ],
        );
        // initialize pool factory
        let pool_hash = e.deployer().upload_contract_wasm(POOL_WASM);
        let pool_init_meta = PoolInitMeta {
            backstop: backstop_id.clone(),
            pool_hash: pool_hash.clone(),
            blnd_id: blnd_id.clone(),
        };
        let pool_factory_client = create_pool_factory(&e, &pool_factory_id, pool_init_meta);

        // drop tokens to admin
        backstop_client.drop();

        // Initialize oracle
        let mock_oracle_client = create_mock_oracle(&e, &mock_oracle_id);
        mock_oracle_client.set_data(
            &admin,
            &Asset::Other(Symbol::new(&e, "USD")),
            &svec![
                &e,
                Asset::Stellar(usdc_id),
                Asset::Stellar(xlm_id.clone()),
                Asset::Other(Symbol::new(&e, "USD")),
                Asset::Other(Symbol::new(&e, "EURO")),
                Asset::Other(Symbol::new(&e, "GBP")),
            ],
            &14,
            &300,
        );
        mock_oracle_client.set_price_stable(&svec![
            &e,
            1_00_000_000_000_000,    // usdc
            0_10_000_000_000_000,    // xlm
            1_00_000_000_000_000,    // usd
            1_10_000_000_000_000,    // euro
            1_20_000_000_000_000,    // gbp
        ]);

        // Initialize soroswap
        let pair_factory_client= create_pair_factory(&e, &pair_factory_id);
        let pair_hash = e.deployer().upload_contract_wasm(PAIR_WASM);
        let router_client = create_router(&e, &router_id);
        pair_factory_client.initialize(&admin, &pair_hash);
        router_client.initialize(&pair_factory_id);

        // Deploy orbit dependencies
        let dao_utils_client = create_dao_utils(&e, &dao_utils_id, wasm);
        let bridge_oracle_client = create_bridge_oracle(&e, &bridge_oracle_id, wasm, &admin, &mock_oracle_id, &mock_oracle_id);
        let treasury_client = create_treasury(&e, &treasury_id, wasm, &admin, &pool_factory_id, &pegkeeper_id);
        let pegkeeper_client = create_pegkeeper(&e, &pegkeeper_id, wasm, &treasury_id, &router_id);

        let fixture = TestFixture {
            env: e,
            admin,
            users: vec![frodo],
            emitter: emitter_client,
            backstop: backstop_client,
            pool_factory: pool_factory_client,
            pair_factory: pair_factory_client,
            router: router_client,
            oracle: mock_oracle_client,
            bridge_oracle: bridge_oracle_client,
            lp: lp_client,
            pools: vec![],
            pairs: vec![],
            tokens: vec![
                blnd_client,
                usdc_client,
                xlm_client,
                ousd_client
            ],
            dao_utils: dao_utils_client,
            treasury: treasury_client,
            pegkeeper: pegkeeper_client,
        };
        fixture.jump(7 * 24 * 60 * 60);
        fixture
    }

    pub fn create_pool(&mut self, name: String, backstop_take_rate: u32, max_positions: u32, min_collateral: i128) {
        let pool_id = self.pool_factory.deploy(
            &self.admin,
            &name,
            &BytesN::<32>::random(&self.env),
            &self.bridge_oracle.address.clone(),
            &backstop_take_rate,
            &max_positions,
            &min_collateral,
        );
        self.pools.push(PoolFixture {
            pool: PoolClient::new(&self.env, &pool_id),
            reserves: HashMap::new(),
        });
    }

    pub fn create_pair(&mut self, token_a: TokenIndex, token_b: TokenIndex, supply_a: i128, supply_b: i128) {
        let token_a_id = &self.tokens[token_a].address;
        let token_b_id = &self.tokens[token_b].address;
        let pair_id = self.pair_factory.create_pair(token_a_id, token_b_id);
        let pair = PairClient::new(&self.env, &pair_id);
        self.tokens[token_a].mint(&pair_id, &supply_a);
        self.tokens[token_b].mint(&pair_id, &supply_b);
        pair.deposit(&self.admin);
        self.pairs.push(pair);
    }

    pub fn create_pool_reserve(
        &mut self,
        pool_index: usize,
        asset_index: TokenIndex,
        reserve_config: &ReserveConfig,
    ) {
        let mut pool_fixture = self.pools.remove(pool_index);
        let token = &self.tokens[asset_index];
        pool_fixture
            .pool
            .queue_set_reserve(&token.address, reserve_config);
        let index = pool_fixture.pool.set_reserve(&token.address);
        pool_fixture.reserves.insert(asset_index, index);
        self.pools.insert(pool_index, pool_fixture);
    }

    /********** Contract Data Helpers **********/

    pub fn read_pool_config(&self, pool_index: usize) -> PoolConfig {
        let pool_fixture = &self.pools[pool_index];
        self.env.as_contract(&pool_fixture.pool.address, || {
            self.env
                .storage()
                .instance()
                .get(&Symbol::new(&self.env, "Config"))
                .unwrap()
        })
    }

    pub fn read_pool_emissions(&self, pool_index: usize) -> Map<u32, u64> {
        let pool_fixture = &self.pools[pool_index];
        self.env.as_contract(&pool_fixture.pool.address, || {
            self.env
                .storage()
                .persistent()
                .get(&Symbol::new(&self.env, "PoolEmis"))
                .unwrap()
        })
    }

    pub fn read_reserve_config(&self, pool_index: usize, asset_index: TokenIndex) -> ReserveConfig {
        let pool_fixture = &self.pools[pool_index];
        let token = &self.tokens[asset_index];
        self.env.as_contract(&pool_fixture.pool.address, || {
            let token_id = &token.address;
            self.env
                .storage()
                .persistent()
                .get(&PoolDataKey::ResConfig(token_id.clone()))
                .unwrap()
        })
    }

    pub fn read_reserve_data(&self, pool_index: usize, asset_index: TokenIndex) -> ReserveData {
        let pool_fixture = &self.pools[pool_index];
        let token = &self.tokens[asset_index];
        self.env.as_contract(&pool_fixture.pool.address, || {
            let token_id = &token.address;
            self.env
                .storage()
                .persistent()
                .get(&PoolDataKey::ResData(token_id.clone()))
                .unwrap()
        })
    }

    /********** Chain Helpers ***********/

    pub fn jump(&self, time: u64) {
        self.env.ledger().set(LedgerInfo {
            timestamp: self.env.ledger().timestamp().saturating_add(time),
            protocol_version: 22,
            sequence_number: self.env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 999999,
            min_persistent_entry_ttl: 999999,
            max_entry_ttl: 9999999,
        });
    }

    pub fn jump_with_sequence(&self, time: u64) {
        let blocks = time / 5;
        self.env.ledger().set(LedgerInfo {
            timestamp: self.env.ledger().timestamp().saturating_add(time),
            protocol_version: 22,
            sequence_number: self.env.ledger().sequence().saturating_add(blocks as u32),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 999999,
            min_persistent_entry_ttl: 999999,
            max_entry_ttl: 9999999,
        });
    }
}
