use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

use crate::{IncreasePositionRequest, LimitOrderParameters};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MintLpParams {
    pub min_out: Option<U128>,
    pub referrer_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapParams {
    pub output_token_id: String,
    pub min_out: Option<U128>,
    pub referrer_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", tag = "action", content = "params")]
#[non_exhaustive]
pub enum Action {
    /// Swap from a fungible token to another fungible token or NEAR
    Swap(SwapParams),

    /// Mint LP tokens from a fungible token
    MintLp(MintLpParams),

    /// Increase position when paying collateral with a fungible token
    IncreasePosition(IncreasePositionRequest),
    PlaceLimitOrder(LimitOrderParameters),
}
