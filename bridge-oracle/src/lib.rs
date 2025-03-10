#![no_std]
#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
mod storage;
#[cfg(test)]
mod test;

pub use contract::*;

