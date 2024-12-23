use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitializedError = 1501,
    InvalidAmount = 1502,
    FlashloanFailedError = 1503,
    NotEnoughSupplyError = 1504,
    BlendPoolNotFoundError = 1506,
    AlreadyAddedError = 1507,
    InvalidBlendPoolError = 1508,
}