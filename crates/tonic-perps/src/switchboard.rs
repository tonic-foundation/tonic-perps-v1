use near_sdk::ext_contract;
use serde::Serialize;

use crate::SwitchboardAddress;

pub const TGAS: u64 = 1_000_000_000_000;

/// Struct expected by Switchboard's `aggregator_read` function.
/// In Switchboard, an aggregator is a feed, in our case containing
/// the prices of assets. When we make a request to `aggregator_read`,
/// we need to specify the `address` of the aggregator (feed). In
/// Switchboard's example, `payer` is set at the same address as
/// `address`.
#[derive(Serialize)]
pub struct Ix {
    /// Aggregator address
    pub address: SwitchboardAddress,

    /// Switchboard sets this to the same thing as address
    /// in their example. Redundant ?
    pub payer: SwitchboardAddress,
}

impl Ix {
    pub fn new(address: SwitchboardAddress) -> Self {
        Ix {
            payer: address,
            address,
        }
    }
}

// Validator interface, for cross-contract calls
#[ext_contract(switchboard_contract)]
trait Switchboard {
    fn aggregator_read(&self, ix: Ix) -> sbv2_near::aggregator::AggregatorRound;
}
