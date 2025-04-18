use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PegkeeperError {
    NotProfitable = 1505,
    PositionStillOpen = 1507,
}