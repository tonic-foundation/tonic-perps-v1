use std::fmt::{self, Display};

use near_sdk::json_types::{I128, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

use crate::json::Base58VecU8;

pub type PositionIdRaw = Base58VecU8;
pub type LimitOrderIdRaw = Base58VecU8;

pub fn emit_event(data: EventType) {
    #[cfg(not(feature = "no_emit"))]
    env::log_str(&Event { data }.to_string());
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Event {
    #[serde(flatten)] // due to tagging options, this adds a "type" key and a "data" key
    pub data: EventType,
}

// we tag this with type/content and flatten it into the event struct. this is
// because serde sometimes has trouble figuring out which enum member the json
// corresponds to
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", tag = "type", content = "data")]
pub enum EventType {
    Swap(SwapEvent),
    MintBurnLp(MintBurnLpEvent),
    EditPosition(EditPositionEvent),
    LiquidatePosition(LiquidatePositionEvent),
    UpdatePosition(UpdatePositionEvent),
    UpdateFundingRate(UpdateFundingRateEvent),
    UpdateProfitLoss(UpdateProfitLossEvent),
    CreateReferralCode(CreateReferralCodeEvent),
    SetReferralCode(SetReferralCodeEvent),
    SetReferrerTier(SetReferrerTierEvent),
    LpPriceUpdate(LpPriceUpdateEvent),
    OracleUpdate(OracleUpdateEvent),
    PlaceLimitOrder(PlaceLimitOrderEvent),
    RemoveLimitOrder(RemoveLimitOrderEvent),
    EditFees(EditFeesEvent),
    EditPoolBalance(EditPoolBalanceEvent),
    EditReservedAmount(EditReservedAmountEvent),
    EditGuaranteedUsd(EditGuaranteedUsdEvent),
    TokenDepositWithdraw(TokenDepositWithdrawEvent),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditPoolBalanceEvent {
    pub amount_native: u128,
    pub new_pool_balance_native: u128,
    pub increase: bool,
    pub account_id: AccountId,
    pub asset_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditReservedAmountEvent {
    pub amount_native: u128,
    pub new_reserved_amount_native: u128,
    pub increase: bool,
    pub account_id: AccountId,
    pub asset_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditGuaranteedUsdEvent {
    pub amount_usd: u128,
    pub new_guaranteed_usd: u128,
    pub increase: bool,
    pub account_id: AccountId,
    pub asset_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum FeeType {
    Burn,
    Funding,
    Mint,
    Position,
    Swap,
    WithrawFee,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditFeesEvent {
    pub fee_native: u128,
    pub fee_usd: u128,
    pub fee_type: FeeType,
    pub new_accumulated_fees_native: u128,
    pub new_accumulated_fees_usd: u128,
    pub increase: bool,
    pub account_id: AccountId,
    pub asset_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenDepositWithdrawEvent {
    pub amount_native: U128,
    pub deposit: bool,
    pub method: String,
    pub receiver_id: AccountId,
    pub account_id: AccountId,
    pub asset_id: String,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).map_err(|_| fmt::Error)?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "swap")]
pub struct SwapEvent {
    pub account_id: AccountId,
    pub token_in: String,
    pub token_out: String,
    pub amount_in_native: U128,
    pub amount_in_usd: U128,
    pub amount_out_native: U128,
    pub amount_out_usd: U128,
    pub fees_native: U128,
    pub fees_usd: U128,
    pub fee_bps: u32,
    pub referral_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum MintBurnDirection {
    Mint,
    Burn,
}

impl Display for MintBurnDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Burn => f.write_str("burn"),
            Self::Mint => f.write_str("mint"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "mint_lp")]
pub struct MintBurnLpEvent {
    pub direction: MintBurnDirection,
    pub account_id: AccountId,
    pub token_in: String,
    pub amount_in: U128,
    pub token_out: String,
    /// Amount sent out
    pub amount_out: U128,
    /// Fee (in terms of the input token for mint, in terms of the redemption token for burn)
    pub fees: U128,
    /// Fee in terms of USD
    pub fees_usd: U128,
    pub fees_bps: u32,
    /// new price after mint
    pub lp_price_usd: U128,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum EditPositionDirection {
    Increase,
    Decrease,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum EditPositionState {
    /// Position newly created.
    Created,

    /// Position closed after editing.
    Closed,

    /// Position existed and remains open after editing.
    Open,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "edit_position")]
pub struct EditPositionEvent {
    pub direction: EditPositionDirection,
    pub account_id: AccountId,
    pub position_id: PositionIdRaw,
    pub collateral_token: String,
    pub underlying_token: String,
    pub collateral_delta_native: U128,
    pub collateral_delta_usd: U128,
    pub size_delta_usd: U128,
    pub new_size_usd: U128,
    pub is_long: bool,
    pub price_usd: U128,

    /// USD out before fees
    pub usd_out: U128,
    pub total_fee_usd: U128,
    pub margin_fee_usd: U128,
    pub position_fee_usd: U128,
    pub total_fee_native: U128,
    pub margin_fee_native: U128,
    pub position_fee_native: U128,
    pub referral_code: Option<String>,

    /// Total realized PnL up to this point.
    pub realized_pnl_to_date_usd: I128,
    pub adjusted_delta_usd: I128,

    /// State as a result of editing.
    pub state: EditPositionState,

    pub limit_order_id: Option<u128>,
    pub liquidator_id: Option<AccountId>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "liquidate_position")]
pub struct LiquidatePositionEvent {
    pub liquidator_id: AccountId,
    pub owner_id: AccountId,
    pub position_id: PositionIdRaw,
    pub collateral_token: String,
    pub underlying_token: String,
    pub is_long: bool,
    pub size_usd: U128,
    pub collateral_usd: U128,
    pub reserve_amount_delta_native: U128,
    pub liquidation_price_usd: U128,
    pub liquidator_reward_native: U128,
    pub liquidator_reward_usd: U128,
    pub fees_native: U128,
    pub fees_usd: U128,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "update_position")]
pub struct UpdatePositionEvent {
    pub account_id: AccountId,
    pub position_id: PositionIdRaw,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "update_profit_loss")]
pub struct UpdateProfitLossEvent {
    pub account_id: AccountId,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "update_funding_rate")]
pub struct UpdateFundingRateEvent {
    pub token_id: String,
    pub funding_rate: U128,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "create_referral_code")]
pub struct CreateReferralCodeEvent {
    pub account_id: AccountId,
    pub referral_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "set_referral_code")]
pub struct SetReferralCodeEvent {
    pub account_id: AccountId,
    pub referral_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "set_referrer_tier")]
pub struct SetReferrerTierEvent {
    pub account_id: AccountId,
    pub referral_code: String,
    pub tier: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "oracle_update")]
pub struct OracleUpdateEvent {
    pub asset_id: String,
    pub price: U128,
    pub spread_bps: u16,
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "lp_price_update")]
pub struct LpPriceUpdateEvent {
    pub price: U128,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "place_limit_order")]
pub struct PlaceLimitOrderEvent {
    pub account_id: AccountId,
    pub limit_order_id: LimitOrderIdRaw,
    pub collateral_token: String,
    pub underlying_token: String,
    pub order_type: String,
    pub threshold_type: String,
    pub collateral_delta_usd: U128,
    pub attached_collateral_native: U128,
    pub size_delta_usd: U128,
    pub price_usd: U128,
    pub expiry: U128,
    pub is_long: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum RemoveOrderReason {
    /// Liquidator's account
    Expired,
    /// Liquidator's account
    Executed,
    /// In case the limit order cannot be executed anymore
    Invalid,

    Removed,
}

impl Display for RemoveOrderReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RemoveOrderReason::Expired => "expired",
            RemoveOrderReason::Executed => "executed",
            RemoveOrderReason::Invalid => "invalid",
            RemoveOrderReason::Removed => "removed",
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", rename = "remove_limit_order")]
pub struct RemoveLimitOrderEvent {
    pub account_id: AccountId,
    pub underlying_token: String,
    pub limit_order_id: LimitOrderIdRaw,
    pub reason: RemoveOrderReason,
    pub liquidator_id: Option<AccountId>,
}

// TODO:
// Collect Swap Fees
// Collect Margin Fees

// event DirectPoolDeposit(address token, uint256 amount);
//     event IncreasePoolAmount(address token, uint256 amount);
//     event DecreasePoolAmount(address token, uint256 amount);
//     event IncreaseUsdgAmount(address token, uint256 amount);
//     event DecreaseUsdgAmount(address token, uint256 amount);
//     event IncreaseReservedAmount(address token, uint256 amount);
//     event DecreaseReservedAmount(address token, uint256 amount);
//     event IncreaseGuaranteedUsd(address token, uint256 amount);
//     event DecreaseGuaranteedUsd(address token, uint256 amount);

// event AddLiquidity(
//     address account,
//     address token,
//     uint256 amount,
//     uint256 aumInUsdg,
//     uint256 glpSupply,
//     uint256 usdgAmount,
//     uint256 mintAmount
// );

// event RemoveLiquidity(
//     address account,
//     address token,
//     uint256 glpAmount,
//     uint256 aumInUsdg,
//     uint256 glpSupply,
//     uint256 usdgAmount,
//     uint256 amountOut
// );

//     event SetDepositFee(uint256 depositFee);
// event SetIncreasePositionBufferBps(uint256 increasePositionBufferBps);
// event SetReferralStorage(address referralStorage);
// event SetAdmin(address admin);
// event WithdrawFees(address token, address receiver, uint256 amount);
