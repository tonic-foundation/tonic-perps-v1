use near_sdk::json_types::{U128, U64};

use crate::{
    get_funding_fee, near_bindgen, AccountId, AdminRole, Asset, AssetId, AssetView, Balance,
    Base58VecU8, ContractState, LimitOrder, LimitOrderId, LiquidationStatus, LiquidationView,
    OrderType, PositionId, PositionView, Serialize, ThresholdType, VContract, VContractExt,
};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MintBurnFeeView {
    pub asset_id: String,
    pub mint_fee_bps: u16,
    pub burn_fee_bps: u16,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeView {
    pub asset_id: String,
    pub token_amount: U128,
    pub usd: U128,
}

impl FeeView {
    fn new(asset_id: &AssetId, token_amount: Balance, usd: Balance) -> Self {
        Self {
            asset_id: asset_id.clone().into(),
            token_amount: token_amount.into(),
            usd: usd.into(),
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitOrderView {
    pub attached_collateral: U128,
    pub collateral_delta: U128,
    pub collateral_id: String,
    pub expiry: U64,
    pub is_long: bool,
    pub id: String,
    pub order_type: OrderType,
    pub owner: String,
    pub price: U128,
    pub size_delta: U128,
    pub threshold: ThresholdType,
    pub underlying_id: String,
}

impl LimitOrderView {
    pub fn new(lo: &LimitOrder, id: &LimitOrderId) -> Self {
        Self {
            attached_collateral: lo.attached_collateral.into(),
            collateral_delta: lo.collateral_delta.into(),
            collateral_id: lo.collateral_id.into_string(),
            expiry: lo.expiry.into(),
            id: id.to_string(),
            is_long: lo.is_long,
            order_type: lo.order_type,
            owner: lo.owner.to_string(),
            price: lo.price.into(),
            size_delta: lo.size_delta.into(),
            threshold: lo.threshold,
            underlying_id: lo.underlying_id.into_string(),
        }
    }
}

#[derive(Serialize)]
pub struct PositionAccountView {
    pub account_id: AccountId,
    #[serde(flatten)]
    pub position: PositionView,
}

#[near_bindgen]
impl VContract {
    pub fn get_asset_info(&self, asset: String) -> AssetView {
        self.contract().assets.unwrap(&asset.into()).to_view()
    }

    pub fn get_assets(&self) -> Vec<AssetView> {
        let assets = self.contract().get_assets();
        assets.values().map(Asset::to_view).collect()
    }

    pub fn get_oracles(&self) -> Vec<AccountId> {
        self.contract().price_oracles.to_vec()
    }

    pub fn get_liquidators(&self) -> Vec<AccountId> {
        self.contract()
            .admins
            .iter()
            .filter_map(|(admin, role)| {
                if role == AdminRole::Liquidator {
                    Some(admin)
                } else {
                    None
                }
            })
            .collect()
    }

    // cases to consider
    // 1. initialAmount is far from targetAmount, action increases balance slightly => high rebate
    // 2. initialAmount is far from targetAmount, action increases balance largely => high rebate
    // 3. initialAmount is close to targetAmount, action increases balance slightly => low rebate
    // 4. initialAmount is far from targetAmount, action reduces balance slightly => high tax
    // 5. initialAmount is far from targetAmount, action reduces balance largely => high tax
    // 6. initialAmount is close to targetAmount, action reduces balance largely => low tax
    // 7. initialAmount is above targetAmount, nextAmount is below targetAmount and vice versa
    // 8. a large swap should have similar fees as the same trade split into multiple smaller swaps
    // pub fn getFeeBasisPoints(address _token, uint256 _usdgDelta, uint256 _feeBasisPoints, uint256 _taxBasisPoints, bool _increment) external view returns (uint256);

    pub fn whitelisted_token_count(&self) -> u32 {
        self.contract().assets.0.len() as u32
    }

    pub fn is_liquidator(&self, account_id: &AccountId) -> bool {
        self.contract().liquidators.contains(account_id)
    }

    pub fn is_admin(&self, account_id: AccountId) -> bool {
        self.contract().admins.get(&account_id).is_some()
    }

    /// Dollar value of all assets under management.
    pub fn get_total_aum(&self) -> U128 {
        self.contract().get_total_aum().into()
    }

    pub fn get_lp_price(&self) -> U128 {
        self.contract().lp_price().into()
    }

    pub fn get_positions(&self, account_id: AccountId) -> Vec<PositionView> {
        let mut positions: Vec<PositionView> = vec![];
        if let Some(position_ids) = self.contract().position_ids_map.get(&account_id) {
            for position_id in position_ids.iter() {
                if let Some(position) = self.get_position(position_id) {
                    positions.push(position);
                }
            }
        }
        positions
    }

    pub fn get_positions_for_asset(
        &self,
        account_id: AccountId,
        asset_id: String,
    ) -> Vec<PositionView> {
        self.get_positions(account_id)
            .into_iter()
            .filter(|position| position.underlying_id == asset_id)
            .collect()
    }

    pub fn get_position(&self, position_id: &PositionId) -> Option<PositionView> {
        let position = self.contract().positions.get(position_id)?;

        let assets = self.contract().assets.clone();
        let collateral = assets.unwrap(&position.collateral_id.clone().into());
        let underlying = assets.unwrap(&position.underlying_id.clone().into());
        let (has_profit, delta) = self.contract().get_delta(
            &underlying,
            position.size,
            position.average_price,
            position.is_long,
            position.last_increased_time,
        );
        let value = if has_profit {
            position.size + delta
        } else {
            position.size.saturating_sub(delta)
        };

        let liquidation_price =
            self.contract()
                .get_liquidation_price(&position, &collateral, position.is_long);

        let funding_fee = get_funding_fee(
            position.size,
            position.entry_funding_rate,
            collateral.cumulative_funding_rate,
        );

        let view = PositionView {
            size: position.size.into(),
            collateral: position.collateral.into(),
            average_price: position.average_price.into(),
            entry_funding_rate: position.entry_funding_rate.into(),
            reserve_amount: position.reserve_amount.into(),
            last_increased_time: position.last_increased_time,
            value: value.into(),
            is_long: position.is_long,
            liquidation_price,
            underlying_id: position.underlying_id.clone(),
            collateral_id: position.collateral_id.clone(),
            funding_fee: funding_fee.into(),
            id: position_id.to_string(),
        };
        Some(view)
    }

    pub fn get_positions_batch(&self, position_ids: Vec<PositionId>) -> Vec<PositionView> {
        position_ids
            .iter()
            .map(|id| self.get_position(id).unwrap())
            .collect()
    }

    pub fn get_position_value(&self, position_id: &PositionId) -> (bool, U128) {
        let assets = self.contract().assets.clone();
        let position = self.contract().positions.get(position_id).unwrap();
        let underlying = assets.unwrap(&AssetId::from(position.underlying_id.clone()));
        let (has_profit, delta) = self.contract().get_delta(
            &underlying,
            position.size,
            position.average_price,
            position.is_long,
            position.last_increased_time,
        );
        (has_profit, U128(delta))
    }

    pub fn get_position_by_id(&self, position_id: Base58VecU8) -> Option<PositionView> {
        let position_id = PositionId::from(position_id);
        self.get_position(&position_id)
    }

    pub fn get_liquidation_status(&self, position_id: &PositionId) -> LiquidationView {
        let position = self.contract().positions.get(position_id).unwrap();
        let assets = self.contract().assets.clone();
        let collateral = assets.unwrap(&AssetId::from(position.collateral_id.clone()));
        let underlying = assets.unwrap(&AssetId::from(position.underlying_id.clone()));
        let (status, fees, leverage) = self.contract().get_liquidation_status(
            &position,
            &collateral,
            &underlying,
            position.is_long,
            true,
        );

        if let LiquidationStatus::Insolvent(reason) = status {
            LiquidationView {
                insolvent: true,
                max_leverage_exceeded: false,
                leverage,
                margin_fee: fees.total_fee_usd.into(),
                reason: Some(reason),
            }
        } else {
            LiquidationView {
                insolvent: false,
                max_leverage_exceeded: status == LiquidationStatus::MaxLeverageExceeded,
                leverage,
                margin_fee: fees.total_fee_usd.into(),
                reason: None,
            }
        }
    }

    pub fn get_mint_burn_fees(&self, asset_id: String, amount: U128) -> MintBurnFeeView {
        let asset = self.contract().assets.unwrap(&asset_id.clone().into());
        MintBurnFeeView {
            asset_id,
            mint_fee_bps: self.contract().get_mint_fee_bps(&asset, amount.0),
            burn_fee_bps: self.contract().get_burn_fee_bps(&asset, amount.0),
        }
    }

    pub fn get_assets_fee(&self) -> Vec<FeeView> {
        self.contract()
            .assets
            .0
            .values()
            .map(|asset| {
                FeeView::new(
                    &asset.asset_id,
                    asset.accumulated_fees,
                    asset.dollar_value_of(asset.accumulated_fees),
                )
            })
            .collect()
    }

    pub fn get_contract_state(&self) -> ContractState {
        self.contract().state.clone()
    }

    pub fn get_limit_orders(&self, asset_id: String) -> Vec<LimitOrderView> {
        self.contract()
            .limit_orders
            .get(&AssetId::from(asset_id))
            .unwrap()
            .to_entries_vec()
            .iter()
            .map(|e| LimitOrderView::new(&e.1, &e.0))
            .collect()
    }

    pub fn get_user_limit_orders(&self, account_id: &AccountId) -> Vec<LimitOrderView> {
        self.contract().get_user_limit_orders(account_id)
    }

    fn get_limit_order_range_vec(
        &self,
        asset_id: &AssetId,
        price_min: U128,
        price_max: U128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> Vec<LimitOrderView> {
        self.contract()
            .limit_orders
            .get(asset_id)
            .unwrap()
            .get_range(price_min.0, price_max.0, is_long, threshold)
            .map(|e| LimitOrderView::new(e.1, e.0))
            .collect()
    }

    pub fn get_limit_order_range(
        &self,
        asset_id: String,
        price_min: U128,
        price_max: U128,
        is_long: Option<bool>,
        threshold: Option<ThresholdType>,
    ) -> Vec<LimitOrderView> {
        let asset_id = &AssetId::from(asset_id);
        match (is_long, threshold) {
            (Some(is_long), Some(threshold)) => {
                self.get_limit_order_range_vec(asset_id, price_min, price_max, is_long, threshold)
            }
            (Some(is_long), None) => [
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    is_long,
                    ThresholdType::Below,
                ),
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    is_long,
                    ThresholdType::Above,
                ),
            ]
            .concat(),
            (None, Some(threshold)) => [
                self.get_limit_order_range_vec(asset_id, price_min, price_max, true, threshold),
                self.get_limit_order_range_vec(asset_id, price_min, price_max, false, threshold),
            ]
            .concat(),
            (None, None) => [
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    true,
                    ThresholdType::Below,
                ),
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    true,
                    ThresholdType::Above,
                ),
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    false,
                    ThresholdType::Below,
                ),
                self.get_limit_order_range_vec(
                    asset_id,
                    price_min,
                    price_max,
                    false,
                    ThresholdType::Above,
                ),
            ]
            .concat(),
        }
    }

    pub fn get_eligible_orders(&self, asset_id: String, max: Option<u64>) -> Vec<LimitOrderId> {
        self.contract()
            .get_eligible_orders(&AssetId::from(asset_id), max)
    }

    pub fn get_default_stablecoin(&self) -> AssetId {
        self.contract().default_stable_coin.clone().unwrap()
    }

    pub fn get_funding_interval_seconds(&self) -> u32 {
        self.contract().funding_interval_seconds
    }

    pub fn get_total_token_weights(&self) -> u32 {
        self.contract().total_weights
    }

    pub fn get_paginated_positions(
        &self,
        skip: Option<U128>,
        max: Option<U128>,
    ) -> Vec<PositionAccountView> {
        let mut keys: Vec<PositionAccountView> = self
            .contract()
            .positions
            .keys()
            .map(|id| PositionAccountView {
                position: self.get_position(&id).unwrap(),
                account_id: self
                    .contract()
                    .positions
                    .get(&id)
                    .unwrap()
                    .account_id
                    .clone(),
            })
            .collect();
        keys.sort_by_key(|a| a.position.last_increased_time);
        keys.into_iter()
            .skip(skip.unwrap_or(U128::from(0)).0 as usize)
            .take(max.unwrap_or(U128::from(u128::MAX)).0 as usize)
            .collect()
    }
}
