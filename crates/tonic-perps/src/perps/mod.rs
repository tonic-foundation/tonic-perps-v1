use crate::{
    borsh, emit_event, env, get_delta, get_funding_fee, get_next_price, near_bindgen, ratio,
    u128_dec_format, AccountId, Asset, AssetId, Balance, BorshDeserialize, BorshSerialize,
    Contract, Deserialize, DollarBalance, EditPositionDirection, EditPositionEvent,
    EditPositionState, EventType, LiquidatePositionEvent, PerpsState, Serialize, TransferInfo,
    VContract, VContractExt, BPS_DIVISOR, LEVERAGE_MULTIPLIER, LIQUIDATION_LEVERAGE_PERCENT,
    LIQUIDATION_REWARD_PERCENT, MIN_MARGIN_PERCENT, PERCENT_MULTIPLIER,
};
use std::time::Duration;

mod limit_order;
mod limit_order_id;
mod position_id;
pub use limit_order::*;
pub use limit_order_id::*;
use near_sdk::{assert_one_yocto, json_types::U128, log};
pub use position_id::*;
use tonic_perps_sdk::prelude::{FeeType, RemoveOrderReason, TokenDepositWithdrawEvent};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VPosition {
    V1(Position),
}

impl From<VPosition> for Position {
    fn from(versioned: VPosition) -> Self {
        match versioned {
            VPosition::V1(position) => position,
        }
    }
}

impl From<Position> for VPosition {
    fn from(position: Position) -> Self {
        VPosition::V1(position)
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Serialize)]
pub enum LiquidationStatus {
    Ok,
    MaxLeverageExceeded,
    BelowMinLeverage,
    Insolvent(String),
}

pub const MAX_LEVERAGE_MESSAGE: &str = "Position leverage is higher than maximum leverage";
pub const MIN_LEVERAGE_MESSAGE: &str = "Position leverage is lower than minimum leverage";

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidationView {
    pub insolvent: bool,
    pub max_leverage_exceeded: bool,
    pub leverage: u16,
    pub margin_fee: U128,
    pub reason: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Position {
    /// Position size, in USD
    pub size: u128,
    /// Position collateral value, in USD
    pub collateral: u128,
    /// Average entry price
    pub average_price: u128,
    /// Cumulative funding rate when position was opened
    pub entry_funding_rate: u128,

    /// Native amount of the collateral asset reserved in the pool
    /// for this position
    pub reserve_amount: u128,
    /// Realized Profit/Loss
    pub realized_pnl: i128,

    /// Last increased time, in ms
    pub last_increased_time: u64,

    /// Position owner's ID
    pub account_id: AccountId,
    /// Collateral asset ID
    pub collateral_id: String,
    /// Underlying asset ID
    pub underlying_id: String,
    /// Long/Short
    pub is_long: bool,
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidationPrice {
    #[serde(with = "u128_dec_format")]
    pub max_leverage: DollarBalance,
    #[serde(with = "u128_dec_format")]
    pub margin_fees: DollarBalance,
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PositionView {
    pub size: U128,
    pub value: U128,
    pub collateral: U128,
    pub average_price: U128,
    pub entry_funding_rate: U128,
    pub reserve_amount: U128,
    pub last_increased_time: u64,
    pub is_long: bool,
    pub liquidation_price: LiquidationPrice,
    pub collateral_id: String,
    pub underlying_id: String,
    pub funding_fee: U128,

    pub id: String,
}

pub struct FeeResult {
    pub margin_fee_usd: u128,
    pub margin_fee_native: u128,
    pub funding_fee_usd: u128,
    pub funding_fee_native: u128,
    pub total_fee_usd: u128,
    pub total_fee_native: u128,
}

impl FeeResult {
    /// Recalculate position fees according to new total fee USD amount
    /// in case a position is insolvent and position fee exceeds the total available collateral.
    /// Guarantees that the fees don't exceed the collateral.
    /// New total fee USD shouldn't exceed the previous one.
    fn adjust_fees_for_insolvent_position(&self, total_usd: DollarBalance, asset: &Asset) -> Self {
        let total_tokens = asset.from_min_usd_price(total_usd);
        FeeResult {
            margin_fee_usd: total_usd,
            margin_fee_native: total_tokens,
            funding_fee_usd: 0,
            funding_fee_native: 0,
            total_fee_usd: total_usd,
            total_fee_native: total_tokens,
        }
    }
}

impl Drop for Position {
    fn drop(&mut self) {
        if self.size != 0 {
            // env::panic_str("tried to drop nonzero position");
            near_sdk::log!("tried to drop nonzero position");
        }
    }
}

fn get_adjusted_delta(
    size_delta: DollarBalance,
    delta: DollarBalance,
    position_size: DollarBalance,
) -> DollarBalance {
    ratio(size_delta, delta, position_size)
}

impl Contract {
    fn validate_tokens_new_position(
        &self,
        collateral_id: &AssetId,
        underlying_id: &AssetId,
        is_long: bool,
    ) {
        let collateral = self.assets.unwrap(collateral_id);
        let underlying = self.assets.unwrap(underlying_id);
        if is_long {
            assert_eq!(
                collateral.asset_id, underlying.asset_id,
                "Collateral token must equal underlying to increase position"
            );
            assert!(!collateral.stable, "Can not open position on stable coin");
        } else {
            assert!(
                collateral.stable,
                "Must provide stablecoin collateral for short position"
            );
            assert!(!underlying.stable, "Can not short stablecoin");
            assert!(underlying.shortable, "Can not short asset");
        }
    }

    /// Ensure asset price is not stale
    pub fn validate_asset_price(&self, asset_id: &AssetId) {
        let asset = self.assets.unwrap(asset_id);
        assert!(asset.price > 0, "Price should be greater than 0");
        if !asset.stable {
            let timestamp = env::block_timestamp_ms();
            assert!(
                timestamp - asset.last_change_timestamp_ms
                    <= std::time::Duration::from_secs(self.max_staleness_duration_sec).as_millis()
                        as u64,
                "Asset price is too stale"
            );
        }
    }

    /// Find a user's existing position for a given collateral/underlying/direction
    /// if one exists
    pub fn get_position_for_user(
        &self,
        account_id: &AccountId,
        collateral_id: &AssetId,
        underlying_id: &AssetId,
        is_long: bool,
    ) -> Option<PositionId> {
        let position_ids = self.position_ids_map.get(account_id)?;

        for position_id in position_ids.iter() {
            let position = self.positions.get(position_id).unwrap();
            if position.collateral_id == collateral_id.into_string()
                && position.underlying_id == underlying_id.into_string()
                && position.is_long == is_long
            {
                return Some(*position_id);
            }
        }

        None
    }

    pub fn add_position_to_user_map(&mut self, account_id: &AccountId, position_id: PositionId) {
        let mut position_ids = self.position_ids_map.get(account_id).unwrap_or_default();
        position_ids.insert(position_id);
        self.position_ids_map.insert(account_id, &position_ids);
    }

    pub fn remove_position_from_user_map(
        &mut self,
        account_id: &AccountId,
        position_id: PositionId,
    ) {
        let mut position_ids = self.position_ids_map.get(account_id).unwrap();
        position_ids.remove(&position_id);
        if !position_ids.is_empty() {
            self.position_ids_map.insert(account_id, &position_ids);
        } else {
            self.position_ids_map.remove(account_id);
        }
    }

    fn default_short_underlying(&self) -> AssetId {
        self.default_stable_coin
            .as_ref()
            .expect("Contract is not correctly initialized.")
            .clone()
    }

    /// Swap collateral into the underlying asset for longs, swap into the default short stable for shorts
    pub fn swap_collateral(
        &mut self,
        input_amount: Balance,
        input: &AssetId,
        underlying: &AssetId,
        is_long: bool,
    ) -> (AssetId, Balance) {
        let input_asset = self.assets.unwrap(input);

        let token_out = if is_long {
            underlying.clone()
        } else if input_asset.stable {
            input.clone()
        } else {
            self.default_short_underlying()
        };

        let collateral_delta = if input.clone() == token_out {
            input_amount
        } else {
            self.swap(input, &token_out, input_amount, None)
        };

        (token_out, collateral_delta)
    }

    /// Get total profit/loss on a position, accounting for the minimum time to
    /// realize a profit
    pub fn get_delta(
        &self,
        underlying: &Asset,
        position_size: DollarBalance,
        average_price: DollarBalance,
        is_long: bool,
        last_increased_time: u64,
    ) -> (bool, DollarBalance) {
        assert!(average_price > 0, "Average price for position must be > 0");
        let price = match is_long {
            true => underlying.min_price(),
            false => underlying.max_price(),
        };
        let (has_profit, delta) = get_delta(average_price, price, position_size, is_long);

        let min_bps = if env::block_timestamp_ms()
            > last_increased_time
                + Duration::from_secs(self.min_profit_time_seconds).as_millis() as u64
        {
            0
        } else {
            underlying.min_profit_bps
        };
        if has_profit && delta * BPS_DIVISOR <= position_size * min_bps {
            (has_profit, 0)
        } else {
            (has_profit, delta)
        }
    }

    /// Get average price for the position after a change
    fn get_next_average_price(
        &self,
        underlying: &Asset,
        position_size: DollarBalance,
        average_price: DollarBalance,
        is_long: bool,
        next_price: DollarBalance,
        size_delta: DollarBalance,
        last_increased_time: u64,
    ) -> DollarBalance {
        let size = position_size;
        let (has_profit, delta) = self.get_delta(
            underlying,
            size,
            average_price,
            is_long,
            last_increased_time,
        );
        get_next_price(size, size_delta, delta, next_price, has_profit, is_long)
    }

    fn get_fees(
        &self,
        collateral: &Asset,
        size_delta: DollarBalance,
        position_size: DollarBalance,
        entry_funding_rate: u128,
        is_long: bool,
    ) -> FeeResult {
        // Margin or position fee
        let position_fee = self.get_position_fee(size_delta, collateral, is_long);
        // Funding fee
        let funding_fee = get_funding_fee(
            position_size,
            entry_funding_rate,
            collateral.cumulative_funding_rate,
        );
        let fee_usd = position_fee + funding_fee;
        let fee_tokens = collateral.from_min_usd_price(fee_usd);

        FeeResult {
            margin_fee_usd: position_fee,
            margin_fee_native: collateral.from_min_usd_price(position_fee),
            funding_fee_usd: funding_fee,
            funding_fee_native: collateral.from_min_usd_price(funding_fee),
            total_fee_usd: fee_usd,
            total_fee_native: fee_tokens,
        }
    }

    /// Takes position fee, which is fixed as a percentage of the position
    /// and funding fee
    fn collect_margin_fee(
        &mut self,
        collateral: &mut Asset,
        size_delta: DollarBalance,
        position_size: DollarBalance,
        entry_funding_rate: u128,
        is_long: bool,
        account_id: &AccountId,
    ) -> FeeResult {
        let fees = self.get_fees(
            collateral,
            size_delta,
            position_size,
            entry_funding_rate,
            is_long,
        );

        // Add fees separately for logging
        collateral.add_fees(fees.funding_fee_native, FeeType::Funding, &account_id);
        collateral.add_fees(fees.margin_fee_native, FeeType::Position, &account_id);

        fees
    }

    fn get_liquidation_price_from_delta(
        &self,
        liquidation_amount: DollarBalance,
        position: &Position,
        is_long: bool,
    ) -> DollarBalance {
        if liquidation_amount > position.collateral {
            let liquidation_delta = liquidation_amount - position.collateral;
            let price_delta = ratio(liquidation_delta, position.average_price, position.size);

            if is_long {
                position.average_price + price_delta
            } else {
                position.average_price - price_delta
            }
        } else {
            let liquidation_delta = position.collateral - liquidation_amount;
            let price_delta = ratio(liquidation_delta, position.average_price, position.size);

            if is_long {
                position.average_price - price_delta
            } else {
                position.average_price + price_delta
            }
        }
    }

    /// Calculate liquidation reward from remaining collateral.
    /// Compare it with max possible reward and return the min value.
    fn get_liquidation_reward(&self, collateral: DollarBalance) -> DollarBalance {
        u128::min(
            ratio(collateral, LIQUIDATION_REWARD_PERCENT, PERCENT_MULTIPLIER),
            self.liquidation_reward_usd,
        )
    }

    /// Calculate liquidation price for a position
    pub fn get_liquidation_price(
        &self,
        position: &Position,
        collateral: &Asset,
        is_long: bool,
    ) -> LiquidationPrice {
        let size = position.size;

        let mut margin_fees = get_funding_fee(
            size,
            position.entry_funding_rate,
            collateral.cumulative_funding_rate,
        );
        margin_fees += self.get_position_fee(size, collateral, position.is_long);
        margin_fees += self.get_liquidation_reward(position.collateral - margin_fees);

        let liquidation_price_for_fees =
            self.get_liquidation_price_from_delta(margin_fees, position, is_long);

        let liquidation_price_for_max_leverage = self.get_liquidation_price_from_delta(
            ratio(size, LEVERAGE_MULTIPLIER, self.max_leverage),
            position,
            is_long,
        );

        LiquidationPrice {
            max_leverage: liquidation_price_for_max_leverage,
            margin_fees: liquidation_price_for_fees,
        }
    }

    /// Checks if a position is ok, exceeds max leverage, or totally insolvent (losses/fees exceed posted collateral)
    /// Return 0 as position leverage if losses exceed collateral.
    pub fn get_liquidation_status(
        &self,
        position: &Position,
        collateral: &Asset,
        underlying: &Asset,
        is_long: bool,
        is_liquidation: bool,
    ) -> (LiquidationStatus, FeeResult, u16) {
        let (has_profit, delta) = self.get_delta(
            underlying,
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
        );

        let fees = self.get_fees(
            collateral,
            position.size,
            position.size,
            position.entry_funding_rate,
            is_long,
        );

        if !has_profit && position.collateral < delta {
            return (
                LiquidationStatus::Insolvent("Losses exceed collateral".to_string()),
                fees,
                0,
            );
        }

        let remaining_collateral = if has_profit {
            position.collateral
        } else {
            position.collateral - delta
        };
        if remaining_collateral < fees.total_fee_usd {
            return (
                LiquidationStatus::Insolvent("Vault: fees exceed collateral".to_string()),
                fees.adjust_fees_for_insolvent_position(remaining_collateral, collateral),
                0,
            );
        }

        let margin_level = ratio(
            remaining_collateral,
            PERCENT_MULTIPLIER,
            position.collateral,
        );
        if margin_level < MIN_MARGIN_PERCENT as u128 {
            return (
                LiquidationStatus::Insolvent("Margin level is less than allowed".to_string()),
                fees,
                0,
            );
        }

        let (status, leverage) = self.check_leverage(
            position.size,
            remaining_collateral - fees.total_fee_usd,
            is_liquidation,
        );

        (status, fees, leverage)
    }

    /// Reduces collateral from a position, taking into account the trader's desired
    /// collateral reduction as well as profits/losses on the trade. Returns:
    /// 1) Profit/Loss proportional to change in position size
    /// 2) Dollar amount to send back (before fees)
    /// 3) Dollar collateral amount to reduce by
    fn reduce_collateral(
        &self,
        position: &Position,
        size_delta: DollarBalance,
        collateral_delta: DollarBalance,
        delta: DollarBalance,
        has_profit: bool,
    ) -> (DollarBalance, DollarBalance, DollarBalance) {
        let mut usd_out: DollarBalance = 0;
        let mut collateral_reduction: DollarBalance = 0;
        let adjusted_delta = get_adjusted_delta(size_delta, delta, position.size);

        if has_profit && adjusted_delta > 0 {
            usd_out = adjusted_delta;
        }

        if !has_profit && adjusted_delta > 0 {
            // Subtract user's loss from the position collateral,
            // panic if losses exceed it, this position should be
            // liquidated in such case.
            collateral_reduction += adjusted_delta;
            assert!(
                collateral_reduction <= position.collateral,
                "Losses exceed collateral"
            );
        }

        if collateral_delta > 0 {
            // In case user wants to withdraw some amount, check that
            // it is less than available collateral amount. If it is not,
            // set it as remained collateral. This check guarantees that
            // total collateral reduction won't exceed posiiton collateral
            let collateral_delta = if collateral_delta <= position.collateral - collateral_reduction
            {
                collateral_delta
            } else {
                position.collateral - collateral_reduction
            };
            // If collateral is being removed add that in
            usd_out += collateral_delta;
            collateral_reduction += collateral_delta;
            log!(
                "collateral delta > 0 usd_out: {} collateral reduction: {}",
                usd_out,
                collateral_reduction
            );
        }

        if position.size == size_delta {
            // If the position will be closed, transfer the remaining collateral out
            usd_out += position.collateral - collateral_reduction;
            collateral_reduction = position.collateral;
            log!(
                "position size = size delta, usd_out: {} collateral reduction: {}",
                usd_out,
                position.collateral
            );
        }

        log!(
            "usd_out: {} adjusted delta: {} has_profit {}",
            usd_out,
            adjusted_delta,
            has_profit
        );

        (adjusted_delta, usd_out, collateral_reduction)
    }

    // explicitly insert position so it can be zeroed before dropping
    fn insert_position(&mut self, id: &PositionId, mut p: Position) {
        assert!(p.size != 0, "tried to save position with zero size");
        self.positions.insert(id, &p);
        p.size = 0;
    }

    fn check_open_interest(&self, size: u128, asset_id: &AssetId, is_long: bool) {
        let asset = self.assets.unwrap(asset_id);
        if is_long && size + asset.global_long_size > asset.open_interest_limits.long {
            env::panic_str("Too much open interest for longs");
        }
        if size + asset.global_short_size > asset.open_interest_limits.short {
            env::panic_str("Too much open interest for shorts");
        }
    }

    fn check_position_size(
        &self,
        underlying: &Asset,
        size: u128,
        is_long: bool,
    ) -> Result<(), &str> {
        let (min, max) = if is_long {
            (
                underlying.position_limits.long.min,
                underlying.position_limits.long.max,
            )
        } else {
            (
                underlying.position_limits.short.min,
                underlying.position_limits.short.max,
            )
        };
        if size > max {
            return Err("Position size is higher than maximum position size");
        }
        if size < min {
            return Err("Position size is lower than minimum position size");
        }
        Ok(())
    }

    fn get_max_leverage(&self, is_liquidation: bool) -> u16 {
        self.max_leverage
            + if is_liquidation {
                ratio(
                    self.max_leverage,
                    LIQUIDATION_LEVERAGE_PERCENT,
                    PERCENT_MULTIPLIER,
                ) as u16
            } else {
                0
            }
    }

    fn check_leverage(
        &self,
        size: DollarBalance,
        collateral: DollarBalance,
        is_liquidation: bool,
    ) -> (LiquidationStatus, u16) {
        let max_leverage = self.get_max_leverage(is_liquidation);
        let position_leverage = ratio(size, LEVERAGE_MULTIPLIER, collateral) as u16;
        let status = if position_leverage > max_leverage {
            LiquidationStatus::MaxLeverageExceeded
        } else if position_leverage < self.min_leverage {
            LiquidationStatus::BelowMinLeverage
        } else {
            LiquidationStatus::Ok
        };
        (status, position_leverage)
    }

    fn check_liquidation_status(
        &self,
        position: &Position,
        collateral: &Asset,
        underlying: &Asset,
        is_liquidation: bool,
    ) {
        let (current_status, _) =
            self.check_leverage(position.size, position.collateral, is_liquidation);
        match current_status {
            LiquidationStatus::MaxLeverageExceeded => env::panic_str(MAX_LEVERAGE_MESSAGE),
            LiquidationStatus::BelowMinLeverage => env::panic_str(MIN_LEVERAGE_MESSAGE),
            _ => (),
        };

        let (status, _, _) = self.get_liquidation_status(
            position,
            collateral,
            underlying,
            position.is_long,
            is_liquidation,
        );

        match status {
            LiquidationStatus::Insolvent(msg) => env::panic_str(&msg),
            LiquidationStatus::MaxLeverageExceeded => env::panic_str(MAX_LEVERAGE_MESSAGE),
            LiquidationStatus::BelowMinLeverage => env::panic_str(MIN_LEVERAGE_MESSAGE),
            _ => (),
        };
    }

    /// Create a position or increase an existing one
    pub fn increase_position(
        &mut self,
        account_id: &AccountId,
        collateral_id: &AssetId,
        underlying_id: &AssetId,
        attached_amount: Balance,
        size_delta: DollarBalance,
        is_long: bool,
        limit_order_id: Option<LimitOrderId>,
    ) -> (PositionId, TransferInfo) {
        self.assert_leverage_enabled();

        let (collateral_id, collateral_delta) =
            self.swap_collateral(attached_amount, collateral_id, underlying_id, is_long);

        self.validate_tokens_new_position(&collateral_id, underlying_id, is_long);
        self.validate_asset_price(&collateral_id);
        self.validate_asset_price(underlying_id);

        let mut underlying = self.assets.unwrap(underlying_id);
        let mut collateral = self.assets.unwrap(&collateral_id);
        assert!(collateral.state.perps.check(PerpsState::Enabled));
        assert!(underlying.state.perps.check(PerpsState::Enabled));

        let collateral_delta_usd = collateral.to_min_usd_price(collateral_delta);
        let collateral_cumulative_funding_rate =
            self.update_cumulative_funding_rate(&mut collateral);

        self.check_open_interest(size_delta, underlying_id, is_long);

        let reserve_delta = collateral.from_max_usd_price(size_delta);

        if is_long {
            if reserve_delta > collateral.available_liquidity() {
                env::panic_str("Not enough reserve to allow the long position");
            }
        } else {
            if size_delta > collateral.available_liquidity() {
                env::panic_str("Not enough reserve to allow the short position");
            }
        }

        let position_id =
            self.get_position_for_user(account_id, &collateral_id, underlying_id, is_long);

        let position_id = position_id.unwrap_or(PositionId::new(
            account_id,
            &collateral_id,
            underlying_id,
            is_long,
            env::block_height(),
        ));
        let position = self.positions.remove(&position_id);

        let price = match is_long {
            true => underlying.max_price(),
            false => underlying.min_price(),
        };

        let (mut new_position, is_new) = match position {
            Some(mut position) => {
                if size_delta > 0 {
                    position.average_price = self.get_next_average_price(
                        &underlying,
                        position.size,
                        position.average_price,
                        is_long,
                        price,
                        size_delta,
                        position.last_increased_time,
                    );
                }
                (position, false)
            }
            None => {
                let position = Position {
                    size: 0,
                    collateral: 0,
                    average_price: price,
                    entry_funding_rate: 0,
                    reserve_amount: 0,
                    realized_pnl: 0,
                    last_increased_time: 0,

                    collateral_id: collateral_id.clone().into(),
                    underlying_id: underlying_id.clone().into(),
                    account_id: account_id.clone(),
                    is_long,
                };
                self.add_position_to_user_map(account_id, position_id);
                (position, true)
            }
        };

        let owner_id = new_position.account_id.clone();
        let fees = self.collect_margin_fee(
            &mut collateral,
            size_delta,
            new_position.size,
            new_position.entry_funding_rate,
            is_long,
            &owner_id,
        );

        collateral.register_deposit(collateral_delta);

        new_position.collateral += collateral_delta_usd;
        if new_position.collateral < fees.total_fee_usd {
            env::panic_str("Position collateral is less than the fee");
        }
        new_position.collateral -= fees.total_fee_usd;

        // getEntryFundingRate always returns the cumulative funding rate of the collateral token
        new_position.entry_funding_rate = collateral_cumulative_funding_rate;
        new_position.size += size_delta;
        new_position.last_increased_time = env::block_timestamp_ms();

        if new_position.size == 0 {
            env::panic_str("position increase must be greater than 0");
        }

        if let Err(message) = self.check_position_size(&underlying, new_position.size, is_long) {
            env::panic_str(message);
        };
        self.check_liquidation_status(&new_position, &collateral, &underlying, false);

        new_position.reserve_amount += reserve_delta;
        collateral.increase_reserved_amount(reserve_delta, &owner_id);

        if is_long {
            // guaranteedUsd stores the sum of (position.size - position.collateral) for all positions
            // if a fee is charged on the collateral then guaranteedUsd should be increased by that fee amount
            // since (position.size - position.collateral) would have increased by `fee`
            collateral.increase_guaranteed_usd(size_delta + fees.total_fee_usd, &owner_id);
            collateral.decrease_guaranteed_usd(collateral_delta_usd, &owner_id);

            // treat the deposited collateral as part of the pool
            collateral.add_liquidity(collateral_delta, &owner_id);
            // fees need to be deducted from the pool since fees are deducted from position.collateral
            collateral.remove_liquidity(fees.total_fee_native, &owner_id);

            // As underlying token equals collateral token, modify it via collateral variable
            collateral.increase_long_size(size_delta);
            collateral.update_long_average_price(price, size_delta);
        } else {
            underlying.update_short_average_price(price, size_delta);
            underlying.increase_short_size(size_delta);
        }

        let new_size_usd = new_position.size;
        let realized_pnl_to_date_usd = new_position.realized_pnl;

        let amount_out = self.update_limit_orders(account_id, &new_position);
        let transfer_data = TransferInfo::new(&owner_id, &collateral_id, amount_out);

        self.insert_position(&position_id, new_position);

        emit_event(EventType::EditPosition(EditPositionEvent {
            direction: EditPositionDirection::Increase,
            state: if is_new {
                EditPositionState::Created
            } else {
                EditPositionState::Open
            },
            account_id: account_id.clone(),
            position_id: position_id.0.to_vec().into(),
            collateral_token: collateral_id.into_string(),
            underlying_token: underlying_id.into_string(),
            collateral_delta_native: collateral_delta.into(),
            collateral_delta_usd: collateral_delta_usd.into(),
            size_delta_usd: size_delta.into(),
            new_size_usd: new_size_usd.into(),
            price_usd: price.into(),
            total_fee_usd: fees.total_fee_usd.into(),
            margin_fee_usd: fees.margin_fee_usd.into(),
            position_fee_usd: fees.funding_fee_usd.into(),
            total_fee_native: fees.total_fee_native.into(),
            margin_fee_native: fees.margin_fee_native.into(),
            position_fee_native: fees.funding_fee_native.into(),
            usd_out: 0.into(),
            realized_pnl_to_date_usd: realized_pnl_to_date_usd.into(),
            adjusted_delta_usd: 0.into(),
            is_long,
            referral_code: self.user_referral_code.get(account_id),
            limit_order_id: limit_order_id.map(|e| e.0),
            liquidator_id: None,
        }));

        self.set_asset(&collateral_id, collateral);
        if !is_long {
            self.set_asset(underlying_id, underlying);
        }

        (position_id, transfer_data)
    }

    /// Decrease a position and send back any profits
    #[must_use]
    pub fn decrease_position(
        &mut self,
        position_id: PositionId,
        collateral_delta: DollarBalance,
        size_delta: DollarBalance,
        limit_order_id: Option<LimitOrderId>,
        is_liquidation: bool,
        output_token_id: Option<String>,
    ) -> TransferInfo {
        let mut position = self.positions.remove(&position_id).unwrap();
        let collateral_id = AssetId::from(position.collateral_id.clone());
        let underlying_id = AssetId::from(position.underlying_id.clone());
        let is_long = position.is_long;
        let owner_id = position.account_id.clone();
        self.validate_asset_price(&collateral_id);
        self.validate_asset_price(&underlying_id);

        let mut underlying = self.assets.unwrap(&underlying_id);
        let mut collateral = self.assets.unwrap(&collateral_id);
        assert!(!underlying.state.perps.check(PerpsState::Disabled));
        assert!(!collateral.state.perps.check(PerpsState::Disabled));

        if limit_order_id.is_some() {
            self.assert_liquidator();
        } else {
            assert!(
                is_liquidation || position.account_id == env::predecessor_account_id(),
                "Can not decrease other account's position"
            );
        }

        assert!(position.size > 0, "Invalid position to decrease");
        if size_delta > position.size {
            env::panic_str("Can not decrease position by more than size");
        }
        if collateral_delta > position.collateral {
            env::panic_str("Can not take more than collateral out of position");
        }

        if is_long {
            collateral.decrease_long_size(size_delta);
        } else {
            underlying.decrease_short_size(size_delta);
        }

        let reserve_delta = ratio(position.reserve_amount, size_delta, position.size);
        position.reserve_amount -= reserve_delta;
        collateral.decrease_reserved_amount(reserve_delta, &owner_id);
        self.update_cumulative_funding_rate(&mut collateral);

        let (has_profit, delta) = self.get_delta(
            &underlying,
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
        );

        let fees = self.collect_margin_fee(
            &mut collateral,
            size_delta,
            position.size,
            position.entry_funding_rate,
            position.is_long,
            &owner_id,
        );

        let (adjusted_delta, usd_out, collateral_reduction) =
            self.reduce_collateral(&position, size_delta, collateral_delta, delta, has_profit);

        collateral.register_withdrawal(collateral.from_min_usd_price(usd_out));

        let mut total_collateral_reduction = collateral_reduction;
        let mut usd_out_after_fee = usd_out;
        // Withdraw fees either from usd_out or from position collateral
        if usd_out > fees.total_fee_usd {
            usd_out_after_fee -= fees.total_fee_usd;
        } else {
            total_collateral_reduction += fees.total_fee_usd;
            // If total collateral reduction exceeds position collateral
            // while usd_out is less than fees, it means that position should be
            // liquidated. If we close a position with a healthy state, there is
            // enough amount of usd_out to withdraw fees from it.
            assert!(
                total_collateral_reduction <= position.collateral,
                "Fees exceed available position collateral"
            );
            if is_long {
                collateral.remove_liquidity(fees.total_fee_native, &owner_id);
            }
        }

        assert!(
            total_collateral_reduction < position.collateral || position.size == size_delta,
            "Not enough collateral to cover losses and send tokens out. 
            Close the position or specify less collateral delta"
        );

        if adjusted_delta > 0 && !is_long {
            let token_amount = collateral.from_min_usd_price(adjusted_delta);
            if has_profit {
                // pay out realised profits from the pool amount for short positions
                collateral.remove_liquidity(token_amount, &owner_id);
            } else {
                // transfer realised losses to the pool for short positions
                // realised losses for long positions are not transferred here as
                // add_liquidity was already called in increase_position for longs
                collateral.add_liquidity(token_amount, &owner_id);
            }
        }

        if adjusted_delta > 0 {
            position.realized_pnl = if has_profit {
                position.realized_pnl + (adjusted_delta as i128)
            } else {
                position.realized_pnl - (adjusted_delta as i128)
            };
        }

        position.size -= size_delta;
        position.collateral -= total_collateral_reduction;
        if position.size > 0 {
            position.entry_funding_rate = collateral.cumulative_funding_rate;
            self.check_liquidation_status(&position, &collateral, &underlying, false);
        }

        if is_long {
            collateral.increase_guaranteed_usd(total_collateral_reduction, &owner_id);
            collateral.decrease_guaranteed_usd(size_delta, &owner_id);
        }

        let amount_out_after_fees = collateral.from_min_usd_price(usd_out_after_fee);
        if usd_out > 0 && is_long {
            collateral.remove_liquidity(collateral.from_min_usd_price(usd_out), &owner_id);
        }

        let account_id = position.account_id.clone();
        let new_size_usd = position.size;
        let is_closed = position.size == 0;
        let realized_pnl_to_date_usd = position.realized_pnl;

        let amount_out = self.update_limit_orders(&account_id, &position) + amount_out_after_fees;

        if !is_closed {
            self.insert_position(&position_id, position);
        } else {
            self.remove_position_from_user_map(&position.account_id, position_id);
        }

        emit_event(EventType::EditPosition(EditPositionEvent {
            direction: EditPositionDirection::Decrease,
            position_id: position_id.0.to_vec().into(),
            state: if is_closed {
                EditPositionState::Closed
            } else {
                EditPositionState::Open
            },
            collateral_token: collateral_id.into_string(),
            underlying_token: underlying_id.into_string(),
            collateral_delta_native: collateral.from_min_usd_price(collateral_delta).into(),
            collateral_delta_usd: collateral_delta.into(),
            size_delta_usd: size_delta.into(),
            new_size_usd: new_size_usd.into(),
            price_usd: match is_long {
                true => underlying.min_price().into(),
                false => underlying.max_price().into(),
            },
            total_fee_usd: fees.total_fee_usd.into(),
            margin_fee_usd: fees.margin_fee_usd.into(),
            position_fee_usd: fees.funding_fee_usd.into(),
            total_fee_native: fees.total_fee_native.into(),
            margin_fee_native: fees.margin_fee_native.into(),
            position_fee_native: fees.funding_fee_native.into(),
            usd_out: usd_out.into(),
            realized_pnl_to_date_usd: realized_pnl_to_date_usd.into(),
            adjusted_delta_usd: if has_profit {
                (adjusted_delta as i128).into()
            } else {
                (-(adjusted_delta as i128)).into()
            },
            is_long,
            referral_code: self.user_referral_code.get(&account_id),
            account_id,
            limit_order_id: limit_order_id.map(|e| e.0),
            liquidator_id: if is_liquidation {
                Some(env::predecessor_account_id())
            } else {
                None
            },
        }));

        self.set_asset(&collateral_id, collateral);
        if !is_long {
            self.set_asset(&underlying_id, underlying);
        }

        if let Some(token_id) = output_token_id {
            let token_id = AssetId::from(token_id);
            if token_id == collateral_id {
                return TransferInfo::new(&owner_id, &collateral_id, amount_out);
            }
            let output_amount = self.swap(&collateral_id, &token_id, amount_out, None);
            return TransferInfo::new(&owner_id, &token_id, output_amount);
        } else {
            TransferInfo::new(&owner_id, &collateral_id, amount_out)
        }
    }

    /// Liquidate insolvent positions or balance positions that exceed [self.max_leverage].
    /// Returns the account ID, collateral asset ID and native token balance
    /// to send either a reward to liquidator or possible profit/remaining tokens
    /// after removing increase limit order to a position owner.
    /// First TransferInfo contains data about position owner's tokens.
    /// Second TransferInfo contains data about liquidator's reward.
    #[must_use]
    pub fn liquidate_position(
        &mut self,
        position_id: PositionId,
    ) -> (LiquidationStatus, TransferInfo, Option<TransferInfo>) {
        let mut position = self.positions.get(&position_id).unwrap();
        let collateral_id = AssetId::from(position.collateral_id.clone());
        let underlying_id = AssetId::from(position.underlying_id.clone());
        let mut collateral = self.assets.unwrap(&collateral_id);
        let mut underlying = self.assets.unwrap(&underlying_id);
        let is_long = position.is_long;
        let owner_id = position.account_id.clone();
        let size_delta = position.size;

        let (has_profit, delta) = self.get_delta(
            &underlying,
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
        );

        if self.private_liquidation_only {
            self.assert_liquidator();
        }

        let (status, fees, _) =
            self.get_liquidation_status(&position, &collateral, &underlying, is_long, true);

        let mark_price = match is_long {
            true => underlying.min_price(),
            false => underlying.max_price(),
        };

        log!(
            "Position liquidation status is {:?} at a price {}",
            status,
            mark_price
        );

        match status {
            LiquidationStatus::Ok => env::panic_str("Position not eligible for liquidation"),
            LiquidationStatus::MaxLeverageExceeded => {
                let owner_transfer_info =
                    self.balance_position_leverage(position_id, &position, fees, has_profit, delta);
                return (status, owner_transfer_info, None);
            }
            _ => {
                self.positions.remove(&position_id).unwrap();
            }
        };

        if is_long {
            collateral.decrease_long_size(position.size);
        } else {
            underlying.decrease_short_size(position.size);
        }

        collateral.add_fees(fees.funding_fee_native, FeeType::Funding, &owner_id);
        collateral.add_fees(fees.margin_fee_native, FeeType::Position, &owner_id);

        collateral.decrease_reserved_amount(position.reserve_amount, &owner_id);
        self.update_cumulative_funding_rate(&mut collateral);

        if is_long {
            collateral.decrease_guaranteed_usd(position.size - position.collateral, &owner_id);
            collateral.remove_liquidity(fees.total_fee_native, &owner_id);
        }

        let remaining_collateral = position.collateral - fees.total_fee_usd;

        if !is_long && fees.total_fee_usd < position.collateral {
            collateral.add_liquidity(
                collateral.from_min_usd_price(remaining_collateral),
                &owner_id,
            );
        }

        let liquidation_reward_usd = self.get_liquidation_reward(remaining_collateral);
        let liquidation_fee_token = collateral.from_min_usd_price(liquidation_reward_usd);
        collateral.remove_liquidity(liquidation_fee_token, &owner_id);

        self.remove_position_from_user_map(&position.account_id, position_id);

        // drop position
        position.size = 0;

        let amount_out = self.update_limit_orders(&position.account_id, &position);
        let liquidator_id = env::predecessor_account_id();
        let owner_transfer_info = TransferInfo::new(&owner_id, &collateral_id, amount_out);
        let liquidator_transfer_info =
            TransferInfo::new(&liquidator_id, &collateral_id, liquidation_fee_token);

        emit_event(EventType::LiquidatePosition(LiquidatePositionEvent {
            owner_id: position.account_id.clone(),
            liquidator_id: liquidator_id.clone(),
            position_id: position_id.0.to_vec().into(),
            collateral_token: collateral_id.into_string(),
            underlying_token: underlying_id.into_string(),
            is_long,
            size_usd: size_delta.into(),
            collateral_usd: position.collateral.into(),
            reserve_amount_delta_native: position.reserve_amount.into(),
            liquidation_price_usd: mark_price.into(),
            liquidator_reward_native: liquidation_fee_token.into(),
            liquidator_reward_usd: collateral.to_min_usd_price(liquidation_fee_token).into(),
            fees_native: fees.total_fee_native.into(),
            fees_usd: fees.total_fee_usd.into(),
        }));

        emit_event(EventType::EditPosition(EditPositionEvent {
            direction: EditPositionDirection::Decrease,
            position_id: position_id.0.to_vec().into(),
            state: EditPositionState::Closed,
            collateral_token: collateral_id.into_string(),
            underlying_token: underlying_id.into_string(),
            collateral_delta_native: collateral.from_min_usd_price(position.collateral).into(),
            collateral_delta_usd: position.collateral.into(),
            size_delta_usd: size_delta.into(),
            new_size_usd: 0.into(),
            price_usd: if is_long {
                self.assets.unwrap(&underlying_id).min_price().into()
            } else {
                self.assets.unwrap(&underlying_id).max_price().into()
            },
            total_fee_usd: fees.total_fee_usd.into(),
            margin_fee_usd: fees.margin_fee_usd.into(),
            position_fee_usd: fees.funding_fee_usd.into(),
            total_fee_native: fees.total_fee_native.into(),
            margin_fee_native: fees.margin_fee_native.into(),
            position_fee_native: fees.funding_fee_native.into(),
            usd_out: 0u128.into(),
            realized_pnl_to_date_usd: (position.realized_pnl - delta as i128).into(),
            adjusted_delta_usd: (-(delta as i128)).into(),
            is_long,
            referral_code: self.user_referral_code.get(&position.account_id.clone()),
            account_id: position.account_id.clone(),
            limit_order_id: None,
            liquidator_id: Some(liquidator_id),
        }));

        self.set_asset(&collateral_id, collateral);
        if !is_long {
            self.set_asset(&underlying_id, underlying);
        }

        (status, owner_transfer_info, Some(liquidator_transfer_info))
    }

    /// Reduce position size so its leverage doesn't exceed [self.max_leverage].
    /// Returns the position owner's account ID, collateral asset ID and native
    /// token balance to send it to the owner if there is a profit.
    /// Note that liquidators currently do not receive any reward for de-leveraging positions.
    fn balance_position_leverage(
        &mut self,
        position_id: PositionId,
        position: &Position,
        fees: FeeResult,
        has_profit: bool,
        delta: DollarBalance,
    ) -> TransferInfo {
        let remained_collateral = if has_profit {
            position.collateral - fees.total_fee_usd
        } else {
            position.collateral - fees.total_fee_usd - delta
        };
        let size_reduction =
            position.size - ratio(remained_collateral, self.max_leverage, LEVERAGE_MULTIPLIER);
        let transfer_info =
            self.decrease_position(position_id, 0, size_reduction, None, true, None);

        transfer_info
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct IncreasePositionRequest {
    pub underlying_id: String,

    /// Dollar amount (DOLLAR_DECIMALS precision) to increase position.
    pub size_delta: U128,

    pub is_long: bool,

    pub referrer_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DecreasePositionRequest {
    pub position_id: PositionId,
    /// Dollar amount (DOLLAR_DECIMALS precision)to decrease position.
    pub size_delta: U128,

    /// Dollar amount (DOLLAR_DECIMALS precision) to reduce collateral.
    pub collateral_delta: U128,

    pub referrer_id: Option<String>,

    /// Preferable token for receiving collateral and profits
    pub output_token_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidatePositionRequest {
    pub position_id: PositionId,
}

#[near_bindgen]
impl VContract {
    /// Increase position with NEAR as collateral. Other collateral payment
    /// requires `ft_transfer_call`.
    #[payable]
    pub fn increase_position(&mut self, params: IncreasePositionRequest) -> PositionId {
        self.contract_mut().assert_running();
        let IncreasePositionRequest {
            is_long,
            size_delta,
            underlying_id,
            referrer_id,
        } = params;
        if let Some(referrer_id) = referrer_id {
            self.set_user_referral_code(referrer_id);
        }
        let contract = self.contract_mut();
        let underlying_id = AssetId::from(underlying_id);
        let (position_id, transfer_info) = contract.increase_position(
            &env::predecessor_account_id(),
            &AssetId::NEAR,
            &underlying_id,
            env::attached_deposit(),
            size_delta.0,
            is_long,
            None,
        );

        contract.internal_send(transfer_info, "remove_limit_order");

        emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
            amount_native: env::attached_deposit().into(),
            deposit: true,
            method: "increase_position".to_string(),
            receiver_id: env::current_account_id(),
            account_id: env::predecessor_account_id(),
            asset_id: AssetId::NEAR.into_string(),
        }));

        position_id
    }

    #[payable]
    pub fn decrease_position(&mut self, params: DecreasePositionRequest) {
        assert_one_yocto();
        self.contract_mut().assert_running();
        if let Some(referrer_id) = params.referrer_id {
            self.set_user_referral_code(referrer_id);
        }
        let contract = self.contract_mut();
        let transfer_info = contract.decrease_position(
            params.position_id,
            params.collateral_delta.0,
            params.size_delta.0,
            None,
            false,
            params.output_token_id,
        );

        contract.internal_send(transfer_info, "decrease_position");
    }

    #[payable]
    pub fn liquidate_position(&mut self, params: LiquidatePositionRequest) -> LiquidationStatus {
        assert_one_yocto();
        let contract = self.contract_mut();
        contract.assert_running();
        let (status, owner_transfer_info, liquidator_transfer_info) =
            self.contract_mut().liquidate_position(params.position_id);

        self.contract()
            .internal_send(owner_transfer_info, "liquidate_position");

        if let Some(liquidator_transfer_info) = liquidator_transfer_info {
            self.contract()
                .internal_send(liquidator_transfer_info, "liquidate_position");
        }

        status
    }

    #[payable]
    pub fn add_limit_order(&mut self, params: LimitOrderParameters) -> LimitOrderId {
        let contract = self.contract_mut();

        contract.assert_limit_order_state(matches!(params.order_type, OrderType::Increase));

        assert!(
            matches!(params.order_type, OrderType::Decrease) || params.collateral_delta.is_none(),
            "collateral_delta field is only required on sell orders"
        );

        assert!(
            (matches!(params.order_type, OrderType::Increase) && params.collateral_id.is_none())
                || (matches!(params.order_type, OrderType::Decrease)
                    && params.collateral_id.is_some()),
            "collateral_id field is only required on sell orders"
        );

        let id = contract.add_limit_order(AddLimitOrderParams {
            owner: env::predecessor_account_id(),
            collateral_id: params.collateral_id.map_or(AssetId::NEAR, AssetId::from),
            underlying_id: AssetId::from(params.underlying_id),
            collateral_delta_usd: params.collateral_delta.unwrap_or(U128(0)).0,
            attached_collateral_native: env::attached_deposit(),
            size: params.size_delta.0,
            price: params.price.0,
            is_long: params.is_long,
            order_type: params.order_type,
            expiry: params.expiry.map(|e| e.0),
        });

        emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
            amount_native: env::attached_deposit().into(),
            deposit: true,
            method: "add_limit_order".to_string(),
            receiver_id: env::current_account_id(),
            account_id: env::predecessor_account_id(),
            asset_id: AssetId::NEAR.into_string(),
        }));

        id
    }

    #[payable]
    pub fn remove_limit_order(&mut self, limit_order_id: LimitOrderId) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        if let Some(transfer_info) = self.contract_mut().remove_limit_order(
            &owner_id,
            &limit_order_id,
            RemoveOrderReason::Removed,
        ) {
            self.contract()
                .internal_send(transfer_info, "remove_limit_order");
        }
    }

    #[payable]
    pub fn execute_limit_order(&mut self, asset_id: String, limit_order_id: LimitOrderId) {
        assert_one_yocto();
        let asset_id = &AssetId::from(asset_id);
        if let Some(transfer_info) = self
            .contract_mut()
            .execute_limit_order(asset_id, &limit_order_id)
        {
            self.contract()
                .internal_send(transfer_info, "execute_limit_order");
        }
    }

    #[payable]
    pub fn remove_outdated_limit_order(&mut self, asset_id: String, limit_order_id: LimitOrderId) {
        assert_one_yocto();
        if let Some(transfer_info) = self
            .contract_mut()
            .remove_outdated_limit_order(&asset_id.into(), &limit_order_id)
        {
            self.contract()
                .internal_send(transfer_info, "remove_outdated_limit_order");
        }
    }
}
