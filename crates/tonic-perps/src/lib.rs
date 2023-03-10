#![allow(clippy::ptr_offset_with_cast, clippy::assign_op_pattern)]
#![deny(warnings)]
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey};
use std::collections::{HashMap, HashSet};

use tonic_perps_sdk::prelude::*;

mod actions;
mod admin;
mod constants;
mod events;
mod fees;
mod lp_token;
mod oracle;
mod perps;
mod referrals;
mod switchboard;
mod token_receiver;
mod token_transfer_history;
mod trading;
mod upgrade;
mod util;
mod vault;
mod views;

pub use actions::*;
pub use admin::*;
pub use constants::*;
pub use events::*;
pub use fees::*;
pub use lp_token::*;
pub use oracle::*;
pub use perps::*;
pub use referrals::*;
pub use token_receiver::*;
pub use token_transfer_history::*;
pub use util::*;
pub use vault::*;
pub use views::*;

/// For example, 1.5x = 1500
pub const LEVERAGE_MULTIPLIER: u16 = 1000;

/// 100%
pub const PERCENT_MULTIPLIER: u16 = 100;

/// Ratio of reward for liquidating a position taken from collateral left. 10%
pub const LIQUIDATION_REWARD_PERCENT: u16 = 10;

/// Ratio for increasing max leverage in order to get position liquidated. 25%
pub const LIQUIDATION_LEVERAGE_PERCENT: u16 = 25;

/// Min margin level to have position opened. 10%
pub const MIN_MARGIN_PERCENT: u16 = 10;

/// Max fee value for any fee parameter. 5%
pub const MAX_FEE_BPS: u16 = 500;

/// Maximum allowed value for reward parameter.
pub const MAX_LIQUIDATION_REWARD_USD: u128 = 100 * DOLLAR_DENOMINATION;

/// Denomination for funding rate, 10_000 = 1%.
pub const FUNDING_RATE_PRECISION: u32 = 1_000_000;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StoragePrefix {
    Account,
    Storage,

    Liquidators,
    Admins,
    Goblins,

    PriceOracles,
    Assets,
    LpToken,

    PositionIdsMap,
    Positions,

    ReferralCodeOwners,
    UserReferralCodes,

    LimitOrderIdsMap,
    LimitOrders,
}

uint::construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub enum VContract {
    V1(Contract),
}

/// Manual implementation of Default for VContract (PanicOnDefault only works on
/// struct definitions)
impl Default for VContract {
    fn default() -> Self {
        env::panic_str("The contract is not initialized");
    }
}

/// Parameters for fee calculations. Can be updated with admin method.
#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeParameters {
    // Fee system is surprisingly complex
    // https://github.com/gmx-io/gmx-contracts/blob/master/contracts/peripherals/Reader.sol#L78
    pub tax_bps: u16,        // = 50; 0.5%
    pub stable_tax_bps: u16, // = 20; 0.2%
    // Base fee for minting or burning TLP
    pub mint_burn_fee_bps: u16, // = 30; 0.3%
    // Base fee for swapping non-stable pairs
    pub swap_fee_bps: u16,        // = 30; 0.3%
    pub stable_swap_fee_bps: u16, // = 4; 0.04%
    pub margin_fee_bps: u16,      // = 10; 0.1%
}

/// perps contract V1
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,

    state: ContractState,

    /// Whitelisted liquidators.
    liquidators: UnorderedSet<AccountId>,

    price_oracles: UnorderedSet<AccountId>,

    /// Admins
    admins: UnorderedMap<AccountId, AdminRole>,

    /// Whitelisted users
    goblins: UnorderedSet<AccountId>,

    /// Whitelisted tokens.
    assets: AssetsMap,

    lp_token: FungibleTokenFreeStorage,

    position_ids_map: UnorderedMap<AccountId, HashSet<PositionId>>,

    positions: UnorderedMap<PositionId, Position>,

    limit_order_ids_map: UnorderedMap<AccountId, HashMap<LimitOrderId, AssetId>>,
    limit_orders: UnorderedMap<AssetId, LimitOrders>,
    limit_order_sequence: u64,

    referral_code_owners: UnorderedMap<String, (AccountId, ReferrerTier)>,
    user_referral_code: UnorderedMap<AccountId, String>,

    /// The sum of weights of each asset in the pool, used as denominator
    /// to calculate target % of each asset
    total_weights: u32,

    /// Fee to pay liquidator in USD. The liquidator receives this
    /// in the underlying collateral of the position, which we will assume
    /// is enough to cover the fee.
    /// https://github.com/gmx-io/gmx-contracts/blob/master/contracts/core/Vault.sol#L748
    /// https://github.com/gmx-io/gmx-contracts/blob/master/contracts/core/VaultUtils.sol#L84
    // liquidation_fee_usd: Balance, // 25 * USD_PRECISION

    // Precision for leverage
    // leverage_multiplier: u32, // 1000
    /// Minimum allowed leverage. 1000 = 1x; see [LEVERAGE_MULTIPLIER].
    min_leverage: u16,

    /// Maximum leverage in a position. 50_000 = 50x; see [LEVERAGE_MULTIPLIER].
    max_leverage: u16, // 50000 = 50x

    /// Reward for liquidating a position in whole numbers of USD. Does not
    /// account for decimals.
    liquidation_reward_usd: u128,

    /// Funding rate interval, default 1h.
    funding_interval_seconds: u32,

    /// Base funding rate in units of [FUNDING_RATE_PRECISION].
    ///
    /// This is scaled by the pool funding rate multiplier when computing
    /// funding payments.
    base_funding_rate: u32,

    // Do we allow spot swaps
    swap_enabled: bool,
    // Do we allow leverage
    leverage_enabled: bool,

    // Do we allow limit orders
    limit_orders_state: LimitOrdersState,

    fee_parameters: FeeParameters,

    // if true only allow "managers" to buy/sell TLP
    manager_mode: bool,

    // Only allow whitelisted liquidators if true
    private_liquidation_only: bool,

    // Minimum amount of time allowed before a user can take a profit
    // on a perp position, to limit MEV frontrunning attacks
    min_profit_time_seconds: u64,

    // Whether to apply multipliers to fees based on pool weights (swap)
    dynamic_swap_fees: bool,

    // Whether to apply multipliers to fees based on pool weights (positions)
    dynamic_position_fees: bool,

    // To be set on initialization
    default_stable_coin: Option<AssetId>,

    // Maximum staleness duration of the price data timestamp.
    max_staleness_duration_sec: u64,

    max_limit_order_life_sec: u64,
}

impl VContract {
    pub fn contract(&self) -> &Contract {
        match self {
            Self::V1(contract) => contract,
        }
    }

    pub fn contract_mut(&mut self) -> &mut Contract {
        match self {
            Self::V1(contract) => contract,
        }
    }
}

#[near_bindgen]
impl VContract {
    #[allow(clippy::new_without_default)]
    #[init]
    #[private]
    pub fn new(owner_id: AccountId) -> Self {
        let mut admins = UnorderedMap::new(StoragePrefix::Admins);
        admins.insert(&owner_id, &AdminRole::FullAdmin);

        Self::V1(Contract {
            owner_id,
            state: ContractState::Paused,

            position_ids_map: UnorderedMap::new(StoragePrefix::PositionIdsMap),
            positions: UnorderedMap::new(StoragePrefix::Positions),

            limit_order_ids_map: UnorderedMap::new(StoragePrefix::LimitOrderIdsMap),
            limit_orders: UnorderedMap::new(StoragePrefix::LimitOrders),
            limit_order_sequence: 0,

            liquidators: UnorderedSet::new(StoragePrefix::Liquidators),
            admins,
            goblins: UnorderedSet::new(StoragePrefix::Goblins),
            price_oracles: UnorderedSet::new(StoragePrefix::PriceOracles),

            referral_code_owners: UnorderedMap::new(StoragePrefix::ReferralCodeOwners),
            user_referral_code: UnorderedMap::new(StoragePrefix::UserReferralCodes),

            assets: AssetsMap(HashMap::new()),
            lp_token: FungibleTokenFreeStorage::new(StoragePrefix::LpToken),

            total_weights: 0,

            min_leverage: LEVERAGE_MULTIPLIER,
            max_leverage: 11 * LEVERAGE_MULTIPLIER,

            liquidation_reward_usd: 25u128 * DOLLAR_DENOMINATION,

            funding_interval_seconds: 60 * 60, // 1 hour
            base_funding_rate: 100,            // 0.01%

            swap_enabled: true,
            leverage_enabled: true,
            limit_orders_state: LimitOrdersState::Enabled,

            manager_mode: false,
            private_liquidation_only: true,

            dynamic_swap_fees: false,
            dynamic_position_fees: false,

            fee_parameters: FeeParameters {
                tax_bps: 50,
                stable_tax_bps: 50,
                mint_burn_fee_bps: 10,
                swap_fee_bps: 10,
                stable_swap_fee_bps: 4,
                margin_fee_bps: 10,
            },

            min_profit_time_seconds: 60,

            default_stable_coin: None,

            max_staleness_duration_sec: 90,

            max_limit_order_life_sec: 60 * 60 * 24 * 30,
        })
    }
}
