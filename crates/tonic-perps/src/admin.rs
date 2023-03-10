use near_sdk::json_types::U128;
use serde::Deserialize;

use crate::{
    asset_parameter, borsh, contract_parameter, env, near_bindgen, require_predecessor, AccountId,
    Asset, AssetId, AssetPositionLimits, AssetState, BorshDeserialize, BorshSerialize, Contract,
    FeeParameters, OpenInterestLimits, Serialize, SwitchboardAddress, VContract, VContractExt,
    LEVERAGE_MULTIPLIER, MAX_FEE_BPS, MAX_LIQUIDATION_REWARD_USD,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractState {
    Running,
    Paused,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AdminRole {
    FullAdmin,
    Liquidator,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, PartialEq, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LimitOrdersState {
    Enabled,
    DecreaseOnly,
    Disabled,
}

impl Contract {
    pub(crate) fn assert_owner(&self) {
        require_predecessor!(self.owner_id, "caller must be owner");
    }

    pub(crate) fn assert_admin(&self) {
        if !self.check_admin_role(AdminRole::FullAdmin) {
            env::panic_str("caller must be approved admin")
        }
    }

    pub(crate) fn assert_liquidator(&self) {
        if !(self.check_admin_role(AdminRole::Liquidator)
            || self.check_admin_role(AdminRole::FullAdmin))
        {
            env::panic_str("caller must be have a Liquidator role")
        }
    }

    pub(crate) fn assert_price_oracle(&self) {
        if !self.price_oracles.contains(&env::predecessor_account_id()) {
            env::panic_str("caller must be approved price oracle")
        }
    }

    pub(crate) fn assert_swap_enabled(&self) {
        assert!(self.swap_enabled, "Swap is currently disabled");
    }

    pub(crate) fn assert_leverage_enabled(&self) {
        assert!(
            self.leverage_enabled,
            "Leverage positions are currently disabled"
        );
    }

    pub(crate) fn assert_running(&self) {
        assert_eq!(
            self.state,
            ContractState::Running,
            "Contract is temporary paused"
        );
    }

    pub fn check_admin_role(&self, role: AdminRole) -> bool {
        if let Some(admin_role) = self.admins.get(&env::predecessor_account_id()) {
            admin_role == role
        } else {
            false
        }
    }

    pub fn assert_limit_order_state(&self, is_increase: bool) {
        if is_increase {
            assert!(
                matches!(self.limit_orders_state, LimitOrdersState::Enabled),
                "Limit orders are disable or only decrease ones are allowed"
            );
        } else {
            assert!(
                !matches!(self.limit_orders_state, LimitOrdersState::Disabled),
                "Limit orders are disabled"
            );
        }
    }
}

#[near_bindgen]
impl VContract {
    pub fn add_admin(&mut self, account_id: AccountId, role: AdminRole) {
        let contract = self.contract_mut();
        contract.assert_owner();
        contract.admins.insert(&account_id, &role);
    }

    pub fn add_price_oracle(&mut self, account_id: AccountId) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.price_oracles.insert(&account_id);
    }

    pub fn remove_admin(&mut self, account_id: AccountId) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.admins.remove(&account_id);
    }

    pub fn remove_price_oracle(&mut self, account_id: AccountId) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.price_oracles.remove(&account_id);
    }

    pub fn set_owner(&mut self, account_id: AccountId) {
        let contract = self.contract_mut();
        contract.assert_owner();
        contract.owner_id = account_id;
    }

    pub fn set_state(&mut self, state: ContractState) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.state = state;
    }

    pub fn set_funding_interval(&mut self, funding_inverval_seconds: u32) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.funding_interval_seconds = funding_inverval_seconds;
    }

    pub fn add_asset(&mut self, asset_id: String, decimals: u8, stable: bool, weight: u32) {
        let contract = self.contract_mut();
        contract.assert_admin();
        assert!(weight > 0, "Asset weight should be positive");

        let asset_id: AssetId = asset_id.into();
        let asset = Asset::new(
            asset_id.clone(),
            decimals,
            stable,
            weight,
            contract.base_funding_rate.into(),
        );
        contract.total_weights += weight;

        contract.assets.insert_new(asset_id, asset);
    }

    pub fn update_asset_weight(&mut self, asset_id: String, weight: u32) {
        assert!(weight > 0, "Asset weight should be positive");
        let contract = self.contract_mut();
        contract.assert_admin();
        let mut asset = contract.assets.unwrap(&asset_id.clone().into());
        contract.total_weights -= asset.token_weight;
        contract.total_weights += weight;
        asset.set_weight(weight);
        contract.set_asset(&asset_id.into(), asset);
    }

    pub fn set_goblins(&mut self, users: Vec<AccountId>) {
        let contract = self.contract_mut();
        contract.assert_admin();

        contract.goblins.clear();
        contract.goblins.extend(users);
    }

    pub fn add_goblins(&mut self, users: Vec<AccountId>) {
        let contract = self.contract_mut();
        contract.assert_admin();
        contract.goblins.extend(users);
    }

    pub fn remove_goblins(&mut self, users: Vec<AccountId>) {
        let contract = self.contract_mut();
        contract.assert_admin();

        for user in users {
            contract.goblins.remove(&user);
        }
    }

    pub fn set_max_asset_price_change(&mut self, asset_id: String, max_change_bps: Option<U128>) {
        let contract = self.contract_mut();
        contract.assert_admin();
        let mut asset = contract.assets.unwrap(&asset_id.clone().into());
        asset.max_price_change_bps = max_change_bps.map(Into::into);
        contract.set_asset(&asset_id.into(), asset);
    }

    pub fn set_default_stablecoin(&mut self, asset_id: String) {
        let contract = self.contract_mut();
        contract.assert_admin();
        assert!(contract.assets.0.get(&asset_id.clone().into()).is_some());
        assert!(contract.assets.unwrap(&asset_id.clone().into()).stable);
        contract.default_stable_coin = Some(asset_id.into());
    }

    pub fn set_withdrawal_limits_settings(
        &mut self,
        asset_id: String,
        sliding_window_duration: Option<u64>,
        withdrawal_limit_bps: Option<U128>,
    ) {
        let contract = self.contract_mut();
        contract.assert_admin();

        let mut asset = contract.assets.unwrap(&asset_id.clone().into());

        if let Some(sliding_window_duration) = sliding_window_duration {
            asset
                .token_transfer_history
                .update_sliding_window_duration(sliding_window_duration);
            asset
                .token_transfer_history
                .clean(env::block_timestamp_ms());
        }

        if let Some(withdrawal_limit_bps) = withdrawal_limit_bps {
            asset.withdrawal_limit_bps = withdrawal_limit_bps.0;
        }
        contract.set_asset(&asset_id.into(), asset);
    }

    pub fn enable_asset(&mut self, asset_id: String) {
        let contract = self.contract_mut();
        contract.assert_admin();
        let mut asset = contract.assets.unwrap(&asset_id.clone().into());
        asset.state.enable_asset();
        contract.set_asset(&asset_id.into(), asset);
    }

    pub fn disable_asset(&mut self, asset_id: String) {
        let contract = self.contract_mut();
        contract.assert_admin();
        let mut asset = contract.assets.unwrap(&asset_id.clone().into());
        asset.state.disable_asset();
        contract.set_asset(&asset_id.into(), asset);
    }
}

asset_parameter!(buffer_amount, U128);
asset_parameter!(max_pool_amount, U128);
asset_parameter!(min_profit_bps, U128);
asset_parameter!(open_interest_limits, OpenInterestLimits);
asset_parameter!(position_limits, AssetPositionLimits);
asset_parameter!(shortable, bool);
asset_parameter!(state, AssetState, |_, _| true, asset_state);
type OptionalSwitchboardAddress = Option<SwitchboardAddress>;
asset_parameter!(switchboard_aggregator_address, OptionalSwitchboardAddress);

contract_parameter!(dynamic_position_fees, bool);
contract_parameter!(dynamic_swap_fees, bool);
contract_parameter!(fee_parameters, FeeParameters, |_, f| {
    f.tax_bps <= MAX_FEE_BPS
        && f.stable_tax_bps <= MAX_FEE_BPS
        && f.mint_burn_fee_bps <= MAX_FEE_BPS
        && f.swap_fee_bps <= MAX_FEE_BPS
        && f.stable_swap_fee_bps <= MAX_FEE_BPS
        && f.margin_fee_bps <= MAX_FEE_BPS
});
contract_parameter!(max_limit_order_life_sec, u64);
contract_parameter!(max_leverage, u16, |contract, max| {
    max > &contract.min_leverage
});
contract_parameter!(max_staleness_duration_sec, u64);
contract_parameter!(min_leverage, u16, |contract, min| {
    contract.max_leverage > *min && *min > LEVERAGE_MULTIPLIER
});
contract_parameter!(min_profit_time_seconds, u64);
contract_parameter!(leverage_enabled, bool);
contract_parameter!(limit_orders_state, LimitOrdersState);
contract_parameter!(liquidation_reward_usd, U128, |_, r| {
    r.0 <= MAX_LIQUIDATION_REWARD_USD
});
contract_parameter!(private_liquidation_only, bool);
contract_parameter!(swap_enabled, bool);
contract_parameter!(base_funding_rate, u32);
