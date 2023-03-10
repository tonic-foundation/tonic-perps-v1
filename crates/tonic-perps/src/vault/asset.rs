use std::str::FromStr;

use near_sdk::{json_types::U128, log};
use tonic_perps_sdk::prelude::{
    emit_event, EditFeesEvent, EditGuaranteedUsdEvent, EditPoolBalanceEvent,
    EditReservedAmountEvent, EventType, FeeType,
};

use crate::{
    borsh, env, ratio, round, u128_dec_format, AccountId, Balance, BorshDeserialize,
    BorshSerialize, Contract, Deserialize, DollarBalance, HashMap, Serialize, SwitchboardAddress,
    TokenTransfer, TokenTransferHistory, TransferType, BN, BPS_DIVISOR, DOLLAR_DENOMINATION,
    FUNDING_RATE_PRECISION, U256,
};

#[derive(
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Hash,
    Clone,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum AssetId {
    NEAR,
    Ft(AccountId),
}

const NEAR_TOKEN_NAME: &str = "near";
const PERCENTAGE_MULTIPLIER: u128 = 100;

impl From<String> for AssetId {
    fn from(s: String) -> Self {
        if s.to_lowercase() == *NEAR_TOKEN_NAME {
            Self::NEAR
        } else {
            Self::Ft(AccountId::from_str(s.as_str()).unwrap())
        }
    }
}

impl From<AssetId> for String {
    fn from(t: AssetId) -> Self {
        match t {
            AssetId::NEAR => NEAR_TOKEN_NAME.to_string(),
            AssetId::Ft(s) => s.to_string(),
        }
    }
}

impl AssetId {
    pub fn into_string(&self) -> String {
        self.clone().into()
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OpenInterestLimits {
    #[serde(with = "u128_dec_format")]
    pub long: u128,
    #[serde(with = "u128_dec_format")]
    pub short: u128,
}

impl Default for OpenInterestLimits {
    fn default() -> Self {
        Self {
            long: u128::MAX,
            short: u128::MAX,
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Limits {
    #[serde(with = "u128_dec_format")]
    pub max: u128,
    #[serde(with = "u128_dec_format")]
    pub min: u128,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max: 2500 * DOLLAR_DENOMINATION,
            min: 0,
        }
    }
}

/// Represents position limits in dollars.
///
/// By default, 0 is the lower limit and 5000
/// the upper limit for both shorts and longs.
///
/// # Examples
///
/// ```
/// use tonic_perps::AssetPositionLimits;
///
/// let apl: AssetPositionLimits = Default::default();
///
/// assert_eq!(0, apl.long.min);
/// assert_eq!(2500_000000, apl.long.max);
/// assert_eq!(0, apl.short.min);
/// assert_eq!(2500_000000, apl.short.max);
/// ```
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetPositionLimits {
    pub long: Limits,
    pub short: Limits,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, PartialEq, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum SwapState {
    Enabled,
    InOnly,
    OutOnly,
    Disabled,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, PartialEq, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PerpsState {
    Enabled,
    ReduceOnly,
    Disabled,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, PartialEq, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LpSupportState {
    Enabled,
    BurnOnly,
    Disabled,
}

impl SwapState {
    pub fn check(&self, state: SwapState) -> bool {
        self == &state
    }
}

impl PerpsState {
    pub fn check(&self, state: PerpsState) -> bool {
        self == &state
    }
}

impl LpSupportState {
    pub fn check(&self, state: LpSupportState) -> bool {
        self == &state
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetState {
    pub swap: SwapState,
    pub perps: PerpsState,
    pub lp_support: LpSupportState,
}

impl AssetState {
    pub fn new() -> Self {
        Self {
            swap: SwapState::Enabled,
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
        }
    }

    pub fn enable_asset(&mut self) {
        self.swap = SwapState::Enabled;
        self.perps = PerpsState::Enabled;
        self.lp_support = LpSupportState::Enabled;
    }

    pub fn disable_asset(&mut self) {
        self.swap = SwapState::Disabled;
        self.perps = PerpsState::Disabled;
        self.lp_support = LpSupportState::Disabled;
    }
}

impl Default for AssetState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    pub asset_id: AssetId, // Native NEAR or FT

    /// Number of decimals
    pub decimals: u8,

    /// Is it a stablecoin
    pub stable: bool,

    /// Token weight in the pool, out of contract.totalTokenWeight
    pub token_weight: u32,

    /// Minimum profit in BPS if it's within the minimum profit time window,
    /// which is set globally for all assets. This is to limit front running
    /// issues
    pub min_profit_bps: Balance,

    /// Maximum amount of pool balance
    pub max_pool_amount: Balance,

    /// Do we allow short perp positions against this asset
    pub shortable: bool,

    /// Total balance of the token in the contract's FT account
    /// ```ignore
    /// balance == pool_balance + fees // should hold, error if not
    /// ```
    pub balance: Balance,

    /// Amount of tokens in the TLP pool
    /// includes posted collateral
    /// Can be used for leverage
    pub pool_balance: Balance,

    /// Amount of this token currently reserved for open leverage positions
    pub reserved_amount: Balance,

    /// Amount to exclude from swaps, in order to ensure a certain amount
    /// of liquidity is available for leveraged positions
    pub buffer_amount: Balance,

    /// Asset state for various operations
    pub state: AssetState,

    /// Total amount short, in USD
    pub global_short_size: DollarBalance,

    /// Average entry price for short positions for this asset
    pub global_short_average_price: DollarBalance,

    /// Total amount long, in USD
    pub global_long_size: DollarBalance,

    /// Average entry price for long positions for this asset
    pub global_long_average_price: DollarBalance,

    /// Amount of fees accumulated for this token type, in the native token
    pub accumulated_fees: Balance,

    /// Price in USD in units of price precision, ie, 1_000_000 = 1 USD.
    ///
    /// In Tonic terms you could say this is the price in "native dollars".
    pub price: DollarBalance,

    /// Spread to charge between buys/sells, in bps difference from index price
    pub spread_bps: u16,

    /// Last time the cumulative funding rate was updated, in ms
    pub last_funding_time: u64,
    /// Maximum funding rate to charge for the asset. Hourly funding is calculated
    /// as utilization % * base_funding_rate
    pub base_funding_rate: u64,
    /// Running total for funding rate
    pub cumulative_funding_rate: u128,

    /// stores the sum of (position.size - position.collateral) for all positions
    pub guaranteed_usd: DollarBalance,

    /// Switchboard address used for asset price feed
    pub switchboard_aggregator_address: Option<SwitchboardAddress>,

    /// The maximum percentage that the asset can change in price from an
    /// update to another. **The amount is in hundreths of a percentage.**
    pub max_price_change_bps: Option<u128>,

    /// Last time the asset price was updated
    pub last_change_timestamp_ms: u64,

    /// Open interest caps for asset
    pub open_interest_limits: OpenInterestLimits,

    /// Maximum position size (per account) for asset
    pub position_limits: AssetPositionLimits,

    /// Struct with a window length (in seconds) and list of withdrawals
    pub token_transfer_history: TokenTransferHistory,

    /// Maximium withdrawal percentage (in bps) that can be withdrawn in the window specified above
    pub withdrawal_limit_bps: u128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetView {
    pub id: String,
    pub decimals: u8,
    pub stable: bool,
    pub shortable: bool,
    pub token_weight: u32,
    pub available_liquidity: U128,
    pub pool_amount: U128,
    pub maximum_pool_amount: U128,
    pub average_price: U128,
    pub entry_price: U128,
    pub exit_price: U128,
    pub funding_rate: u64,
    pub funding_rate_percentage: String,
    pub accumulated_fees: U128,
    pub aum: U128,
    pub position_limits: AssetPositionLimits,
    pub open_interest_long: U128,
    pub open_interest_short: U128,
}

/// Use [Drop] to ensure balance integrity.
impl Drop for Asset {
    fn drop(&mut self) {
        let expected = self.pool_balance + self.accumulated_fees;
        if self.balance != expected {
            log!(
                "balance: {}, pool bal {}, fees {}, discrepancy {}",
                self.balance,
                self.pool_balance,
                self.accumulated_fees,
                if self.balance > expected {
                    self.balance - expected
                } else {
                    expected - self.balance
                }
            );
            env::panic_str("accounting bug: asset balance discrepancy")
        }
    }
}

impl Asset {
    pub fn debug(&self, prefix: &str) {
        let expected = self.pool_balance + self.accumulated_fees;
        println!(
            "DEBUG after: {} balance: {}, pool bal {}, fees {}, discrepancy {}",
            prefix,
            self.balance,
            self.pool_balance,
            self.accumulated_fees,
            if self.balance > expected {
                self.balance - expected
            } else {
                expected - self.balance
            }
        );
    }

    pub fn to_view(&self) -> AssetView {
        let funding_rate = self.current_funding_rate();
        let funding_rate_percentage = (funding_rate as f64 * PERCENTAGE_MULTIPLIER as f64
            / FUNDING_RATE_PRECISION as f64)
            .to_string();

        AssetView {
            id: self.asset_id.clone().into(),
            decimals: self.decimals,
            stable: self.stable,
            shortable: self.shortable,
            token_weight: self.token_weight,
            pool_amount: self.pool_balance.into(),
            available_liquidity: self.available_liquidity().into(),
            average_price: self.price.into(),
            entry_price: self.max_price().into(),
            exit_price: self.min_price().into(),
            accumulated_fees: self.accumulated_fees.into(),
            funding_rate,
            funding_rate_percentage,
            maximum_pool_amount: 0.into(),
            aum: self.aum().into(),
            position_limits: self.position_limits.clone(),
            open_interest_long: self.global_long_size.into(),
            open_interest_short: self.global_short_size.into(),
        }
    }

    pub fn new(
        asset_id: AssetId,
        decimals: u8,
        stable: bool,
        token_weight: u32,
        base_funding_rate: u64,
    ) -> Self {
        Self {
            asset_id,
            decimals,
            stable,
            token_weight,
            base_funding_rate,
            min_profit_bps: 0,
            max_pool_amount: 0,
            shortable: false,
            balance: 0,
            pool_balance: 0,
            reserved_amount: 0,
            buffer_amount: 0,
            state: Default::default(),
            global_short_size: 0,
            global_short_average_price: 0,
            global_long_size: 0,
            global_long_average_price: 0,
            accumulated_fees: 0,
            price: 0,
            spread_bps: 0,
            last_funding_time: 0,
            cumulative_funding_rate: 0,
            guaranteed_usd: 0,
            switchboard_aggregator_address: None,
            max_price_change_bps: None,
            last_change_timestamp_ms: 0,
            open_interest_limits: Default::default(),
            position_limits: Default::default(),
            token_transfer_history: Default::default(),
            withdrawal_limit_bps: 5000,
        }
    }

    pub fn set_shortable(&mut self, shortable: bool) {
        assert!(!self.stable, "Can not set stable as shortable");
        self.shortable = shortable;
    }

    /// Returns native amount for 1 unit of this asset
    pub fn denomination(&self) -> Balance {
        10u128.pow(self.decimals as u32)
    }

    /// Returns native amount of asset available for swaps or new positions
    pub fn available_liquidity(&self) -> Balance {
        if self.pool_balance > self.reserved_amount {
            self.pool_balance - self.reserved_amount
        } else {
            0
        }
    }

    /// Convert native amount of the asset into dollar value.
    pub fn dollar_value_of(&self, amount: Balance) -> DollarBalance {
        ratio(amount, self.price, self.denomination())
    }

    /// Returns dollar value of all assets in the pool + profits/losses from shorts on the asset
    /// with *any* collateral backing those shorts
    pub fn aum(&self) -> DollarBalance {
        if !self.stable {
            let mut aum: DollarBalance = 0;
            let short_size = self.global_short_size;
            let average_price = self.global_short_average_price;
            aum += self.guaranteed_usd;
            aum += self.dollar_value_of(self.pool_balance);
            aum -= self.dollar_value_of(self.reserved_amount);

            if short_size > 0 {
                let (has_profit, delta) = get_delta(average_price, self.price, short_size, false);
                if has_profit {
                    aum.saturating_sub(delta)
                } else {
                    aum + delta
                }
            } else {
                aum
            }
        } else {
            self.dollar_value_of(self.pool_balance)
        }
    }

    /// Add amount to the pool.
    pub fn add_liquidity(&mut self, amount: Balance, account_id: &AccountId) {
        self.balance += amount;
        self.pool_balance += amount;

        if self.max_pool_amount != 0 && self.pool_balance > self.max_pool_amount {
            env::panic_str("Exceed max possible pool amount for this asset");
        }

        emit_event(EventType::EditPoolBalance(EditPoolBalanceEvent {
            amount_native: amount,
            new_pool_balance_native: self.pool_balance,
            increase: true,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    /// Remove amount from the pool.
    pub fn remove_liquidity(&mut self, amount: Balance, account_id: &AccountId) {
        self.balance -= amount;
        self.pool_balance -= amount;
        emit_event(EventType::EditPoolBalance(EditPoolBalanceEvent {
            amount_native: amount,
            new_pool_balance_native: self.pool_balance,
            increase: false,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    /// Minimum price for the asset, accounting for the spread if any
    pub fn min_price(&self) -> DollarBalance {
        BN!(self.price).sub_bps(self.spread_bps).as_u128()
    }

    /// Maximum price for the asset, accounting for the spread if any
    pub fn max_price(&self) -> DollarBalance {
        BN!(self.price).add_bps(self.spread_bps).as_u128()
    }

    /// Add fees, which are not counted as part of the pool
    pub fn add_fees(&mut self, fees: Balance, fee_type: FeeType, account_id: &AccountId) {
        self.balance += fees;
        self.accumulated_fees += fees;
        emit_event(EventType::EditFees(EditFeesEvent {
            fee_native: fees,
            fee_usd: self.to_min_usd_price(fees),
            fee_type,
            new_accumulated_fees_native: self.accumulated_fees,
            new_accumulated_fees_usd: self.to_min_usd_price(self.accumulated_fees),
            increase: true,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn remove_fees(&mut self, fees: Balance, fee_type: FeeType, account_id: &AccountId) {
        self.balance -= fees;
        self.accumulated_fees -= fees;
        emit_event(EventType::EditFees(EditFeesEvent {
            fee_native: fees,
            fee_usd: self.to_min_usd_price(fees),
            fee_type,
            new_accumulated_fees_native: self.accumulated_fees,
            new_accumulated_fees_usd: self.to_min_usd_price(self.accumulated_fees),
            increase: false,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn increase_reserved_amount(&mut self, amount: Balance, account_id: &AccountId) {
        self.reserved_amount += amount;
        emit_event(EventType::EditReservedAmount(EditReservedAmountEvent {
            amount_native: amount,
            new_reserved_amount_native: self.reserved_amount,
            increase: true,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn decrease_reserved_amount(&mut self, amount: Balance, account_id: &AccountId) {
        self.reserved_amount -= amount;
        emit_event(EventType::EditReservedAmount(EditReservedAmountEvent {
            amount_native: amount,
            new_reserved_amount_native: self.reserved_amount,
            increase: false,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn increase_guaranteed_usd(&mut self, amount: DollarBalance, account_id: &AccountId) {
        self.guaranteed_usd += amount;
        emit_event(EventType::EditGuaranteedUsd(EditGuaranteedUsdEvent {
            amount_usd: amount,
            new_guaranteed_usd: self.guaranteed_usd,
            increase: true,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn decrease_guaranteed_usd(&mut self, amount: DollarBalance, account_id: &AccountId) {
        self.guaranteed_usd -= amount;
        emit_event(EventType::EditGuaranteedUsd(EditGuaranteedUsdEvent {
            amount_usd: amount,
            new_guaranteed_usd: self.guaranteed_usd,
            increase: false,
            account_id: account_id.clone(),
            asset_id: self.asset_id.into_string(),
        }));
    }

    pub fn increase_short_size(&mut self, amount: DollarBalance) {
        self.global_short_size += amount;
    }

    pub fn decrease_short_size(&mut self, amount: DollarBalance) {
        self.global_short_size -= amount;
    }

    pub fn increase_long_size(&mut self, amount: DollarBalance) {
        self.global_long_size += amount;
    }

    pub fn decrease_long_size(&mut self, amount: DollarBalance) {
        self.global_long_size -= amount;
    }

    pub fn update_cumulative_funding_rate(
        &mut self,
        block_timestamp_seconds: u64,
        funding_interval_seconds: u64,
    ) -> u128 {
        if self.last_funding_time + funding_interval_seconds > block_timestamp_seconds {
            return self.cumulative_funding_rate;
        }

        let new_funding_rate =
            self.get_next_funding_rate(block_timestamp_seconds, funding_interval_seconds);
        self.cumulative_funding_rate += new_funding_rate as u128;
        self.last_funding_time = round(block_timestamp_seconds, funding_interval_seconds);

        self.cumulative_funding_rate
    }

    pub fn min_funding_rate(&self) -> u64 {
        self.base_funding_rate / 5
    }

    /// Calculates funding rate from pool utilization and base funding rate
    pub fn current_funding_rate(&self) -> u64 {
        if self.pool_balance == 0 {
            return self.min_funding_rate();
        }
        std::cmp::max(
            BN!(self.base_funding_rate)
                .mul(self.reserved_amount)
                .div(self.pool_balance)
                .as_u64(),
            self.min_funding_rate(),
        )
    }

    pub fn get_next_funding_rate(
        &self,
        block_timestamp_seconds: u64,
        funding_interval_seconds: u64,
    ) -> u64 {
        if self.pool_balance == 0 {
            return 0;
        }

        let intervals =
            (block_timestamp_seconds - self.last_funding_time) / funding_interval_seconds;
        let funding_rate = self.current_funding_rate();
        funding_rate * intervals
    }

    pub fn update_long_average_price(
        &mut self,
        next_price: DollarBalance,
        size_delta: DollarBalance,
    ) {
        self.global_long_average_price = self.get_average_price(
            self.global_long_average_price,
            self.global_long_size,
            next_price,
            size_delta,
            true,
        );
    }

    pub fn update_short_average_price(
        &mut self,
        next_price: DollarBalance,
        size_delta: DollarBalance,
    ) {
        self.global_short_average_price = self.get_average_price(
            self.global_short_average_price,
            self.global_short_size,
            next_price,
            size_delta,
            false,
        );
    }

    pub fn get_average_price(
        &self,
        average_price: DollarBalance,
        size: DollarBalance,
        next_price: DollarBalance,
        size_delta: DollarBalance,
        is_long: bool,
    ) -> DollarBalance {
        if average_price == 0 {
            next_price
        } else {
            let (has_profit, delta) = get_delta(average_price, next_price, size, is_long);
            get_next_price(size, size_delta, delta, next_price, has_profit, is_long)
        }
    }

    /// Returns dollar value of the asset amount using the minimum price
    pub fn to_min_usd_price(&self, amount: Balance) -> DollarBalance {
        ratio(amount, self.min_price(), self.denomination())
    }

    /// Returns dollar value of the asset amount using the maximum price
    pub fn to_max_usd_price(&self, amount: Balance) -> DollarBalance {
        ratio(amount, self.max_price(), self.denomination())
    }

    /// Returns asset amount from USD using the maximum price
    pub fn from_max_usd_price(&self, amount: DollarBalance) -> Balance {
        ratio(amount, self.denomination(), self.max_price())
    }

    /// Returns asset amount from USD using the minimum price
    pub fn from_min_usd_price(&self, amount: DollarBalance) -> Balance {
        ratio(amount, self.denomination(), self.min_price())
    }

    /// Returns true if withdrawing `amount` is within the withdrawal limits
    /// for the asset
    pub fn check_withdrawal_limit(&self, amount: Balance) -> bool {
        let amount_in_window = self.token_transfer_history.amount();

        amount_in_window == 0
            || amount + amount_in_window
                < ratio(self.pool_balance, self.withdrawal_limit_bps, BPS_DIVISOR)
    }

    /// Records a withdrawal in the sliding window
    pub fn register_withdrawal(&mut self, amount: Balance) {
        self.token_transfer_history.clean(env::block_timestamp_ms());
        if !self.check_withdrawal_limit(amount) {
            env::panic_str("Exceeded withdrawal limiter");
        }
        self.token_transfer_history.push(TokenTransfer::new(
            amount,
            env::block_timestamp_ms(),
            TransferType::Withdraw,
        ));
    }

    /// Records a deposit in the sliding window
    pub fn register_deposit(&mut self, amount: Balance) {
        self.token_transfer_history.clean(env::block_timestamp_ms());
        self.token_transfer_history.push(TokenTransfer::new(
            amount,
            env::block_timestamp_ms(),
            TransferType::Deposit,
        ));
    }

    pub fn set_buffer_amount(&mut self, amount: Balance) {
        self.buffer_amount = amount;
    }

    pub fn set_weight(&mut self, weight: u32) {
        self.token_weight = weight;
    }

    pub fn set_max_tlp_amount(&mut self, amount: u128) {
        self.max_pool_amount = amount;
    }

    pub fn update_open_interest_limits(&mut self, limits: OpenInterestLimits) {
        self.open_interest_limits = limits;
    }

    pub fn update_position_limits(&mut self, limits: AssetPositionLimits) {
        self.position_limits = limits;
    }

    pub fn check_available_liquidity(&self) {
        assert!(
            self.available_liquidity() >= self.buffer_amount,
            "Vault: poolAmount < buffer"
        );
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct AssetsMap(pub HashMap<AssetId, Asset>);

impl AssetsMap {
    pub fn unwrap(&self, asset_id: &AssetId) -> Asset {
        self.get(asset_id).expect("Asset not found").clone()
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&Asset> {
        self.0.get(asset_id)
    }

    /// Insert a new asset. Panics if duplicate.
    pub fn insert_new(&mut self, asset_id: AssetId, asset: Asset) {
        if self.0.insert(asset_id, asset).is_some() {
            env::panic_str("asset already exists")
        };
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_total_aum(&self) -> DollarBalance {
        self.0.values().map(Asset::aum).sum()
    }
}

impl Contract {
    pub fn set_asset(&mut self, asset_id: &AssetId, asset: Asset) {
        self.assets.0.insert(asset_id.clone(), asset);
    }

    pub fn get_assets(&self) -> HashMap<AssetId, Asset> {
        self.assets.0.clone()
    }

    /// Add amount to the pool.
    pub fn add_liquidity(&mut self, asset_id: &AssetId, amount: Balance) {
        let mut asset = self.assets.unwrap(asset_id);
        asset.add_liquidity(amount, &env::predecessor_account_id());
        self.set_asset(asset_id, asset);
    }
}

pub fn get_delta(average_price: u128, next_price: u128, size: u128, is_long: bool) -> (bool, u128) {
    let price_delta = match average_price > next_price {
        true => average_price - next_price,
        false => next_price - average_price,
    };
    let delta = ratio(size, price_delta, average_price);

    let has_profit = match is_long {
        true => next_price > average_price,
        false => average_price > next_price,
    };
    (has_profit, delta)
}

pub fn get_next_price(
    size: u128,
    size_delta: u128,
    delta: u128,
    next_price: u128,
    has_profit: bool,
    is_long: bool,
) -> u128 {
    let next_size = size + size_delta;
    let divisor = match is_long {
        true if has_profit => next_size + delta,
        true if !has_profit => next_size - delta,
        false if has_profit => next_size - delta,
        false if !has_profit => next_size + delta,
        _ => env::panic_str("cant get here"),
    };
    ratio(next_price, next_size, divisor)
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate::Asset;

    #[test]
    #[should_panic = "asset balance discrepancy"]
    fn asset_integrity() {
        let _ = Asset {
            balance: 2,
            pool_balance: 1,
            reserved_amount: 0,
            state: Default::default(),
            buffer_amount: 0,
            global_short_size: 0,
            global_long_size: 0,

            asset_id: "foo".to_string().into(),
            decimals: 1,
            stable: false,
            token_weight: 1,
            min_profit_bps: 1,
            max_pool_amount: 1,
            shortable: true,
            global_short_average_price: 0,
            global_long_average_price: 0,
            accumulated_fees: 0,
            price: 0,
            spread_bps: 0,

            base_funding_rate: 0,
            last_funding_time: 0,
            cumulative_funding_rate: 0,
            guaranteed_usd: 0,
            switchboard_aggregator_address: None,
            max_price_change_bps: None,
            last_change_timestamp_ms: 0,
            open_interest_limits: Default::default(),
            position_limits: Default::default(),
            token_transfer_history: Default::default(),
            withdrawal_limit_bps: 10000,
        };
    }
}
