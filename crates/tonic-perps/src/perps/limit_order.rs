use near_sdk::json_types::{U128, U64};
use near_sdk::{AccountId, Balance};
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Range;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::ops::Bound::Included;

use crate::{
    borsh, emit_event, env, get_delta, ratio, AssetId, BorshDeserialize, BorshSerialize, Contract,
    DollarBalance, EventType, LimitOrderId, LimitOrderView, LiquidationStatus,
    PlaceLimitOrderEvent, Position, RemoveLimitOrderEvent, RemoveOrderReason, TransferInfo,
};

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy,
)]
pub enum OrderType {
    Increase,
    Decrease,
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            OrderType::Increase => "increase",
            OrderType::Decrease => "decrease",
        })
    }
}

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy,
)]
pub enum ThresholdType {
    Above,
    Below,
}

impl Display for ThresholdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThresholdType::Above => "above",
            ThresholdType::Below => "below",
        })
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
pub struct LimitOrder {
    /// The account creating the order
    pub owner: AccountId,
    /// The amount of collateral in usd for decrease position
    pub collateral_delta: DollarBalance,
    /// The amount of collateral already deposited by the owner
    pub attached_collateral: Balance,
    /// The size delta in usd (by which to increase/decrease a position)
    pub size_delta: DollarBalance,
    /// The ID of the collateral asset
    pub collateral_id: AssetId,
    /// The ID of the underlying asset
    pub underlying_id: AssetId,
    /// The price of the underlying asset at which to execute the order
    pub price: DollarBalance,
    /// Long or short
    pub is_long: bool,
    /// The date when the order should be dropped
    pub expiry: u64,
    /// The type of limit order
    pub order_type: OrderType,
    /// Above or below threshold
    pub threshold: ThresholdType,
}

impl LimitOrder {
    fn new(params: AddLimitOrderParams, threshold: ThresholdType) -> Self {
        Self {
            owner: params.owner,
            collateral_delta: params.collateral_delta_usd,
            attached_collateral: params.attached_collateral_native,
            size_delta: params.size,
            collateral_id: params.collateral_id,
            underlying_id: params.underlying_id,
            price: params.price,
            is_long: params.is_long,
            order_type: params.order_type,
            expiry: params.expiry.unwrap(),
            threshold,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderParameters {
    pub price: U128,      // Dollars
    pub size_delta: U128, // Dollars
    pub collateral_id: Option<String>,
    pub underlying_id: String,
    pub is_long: bool,
    pub order_type: OrderType,
    pub expiry: Option<U64>,
    // If sell only
    pub collateral_delta: Option<U128>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddLimitOrderParams {
    pub owner: AccountId,
    pub collateral_id: AssetId,
    pub underlying_id: AssetId,
    pub collateral_delta_usd: DollarBalance,
    pub attached_collateral_native: Balance,
    pub size: DollarBalance,
    pub price: DollarBalance,
    pub is_long: bool,
    pub order_type: OrderType,
    pub expiry: Option<u64>,
}

impl Contract {
    fn check_limit_order(&self, limit_order: &LimitOrder) {
        assert!(
            limit_order.expiry <= env::block_timestamp_ms() + self.max_limit_order_life_sec * 1000,
            "Max order lifetime exeeded"
        );
        assert!(
            matches!(limit_order.order_type, OrderType::Increase)
                || limit_order.attached_collateral == 0,
            "You cannot attach tokens on a sell order"
        );
        assert!(
            matches!(limit_order.order_type, OrderType::Decrease)
                || limit_order.collateral_delta == 0,
            "You must not provide collateral delta on a buy order"
        );
        assert!(limit_order.is_long || self.assets.get(&limit_order.collateral_id).unwrap().stable);
        assert!(
            !limit_order.is_long || limit_order.underlying_id == limit_order.collateral_id,
            "Collateral token must equal underlying to create limit order for long position"
        );

        let limit_order_collateral_usd = self.get_collateral_in_usd(limit_order);

        assert!(
            limit_order.size_delta > 0 || limit_order_collateral_usd > 0,
            "Can't create a limit order that changes nothing"
        );

        let (new_collateral, new_size): (DollarBalance, DollarBalance) = if let Some(id) = self
            .get_position_for_user(
                &limit_order.owner,
                &limit_order.collateral_id,
                &limit_order.underlying_id,
                limit_order.is_long,
            ) {
            let position = self.positions.get(&id).unwrap();

            let res =
                self.get_position_new_values(limit_order, &position, limit_order_collateral_usd);

            let (new_collateral, new_size) = match res {
                Ok((new_collateral, new_size)) => (new_collateral, new_size),
                Err(message) => env::panic_str(message),
            };

            if new_size == 0 {
                let new_collateral = self.subtract_possible_losses(
                    &position,
                    limit_order,
                    position.size,
                    position.collateral,
                );
                assert!(new_collateral.is_some(), "Losses will exceed collateral");

                return;
            }

            let new_collateral =
                self.subtract_possible_losses(&position, limit_order, new_size, new_collateral);
            assert!(
                new_collateral.is_some(),
                "Losses will exceed remaining collateral"
            );

            (new_collateral.unwrap(), new_size)
        } else {
            assert!(
                matches!(limit_order.order_type, OrderType::Increase),
                "Position not found"
            );
            assert!(
                limit_order.attached_collateral > 0,
                "Cannot create an order without collateral without a previous position"
            );
            (limit_order_collateral_usd, limit_order.size_delta)
        };

        let underlying = self.assets.unwrap(&limit_order.underlying_id);
        if let Err(message) = self.check_position_size(&underlying, new_size, limit_order.is_long) {
            env::panic_str(message);
        };
        match self.check_leverage(new_size, new_collateral, false).0 {
            LiquidationStatus::BelowMinLeverage => {
                env::panic_str("Limit order exceeds minimum leverage")
            }
            LiquidationStatus::MaxLeverageExceeded => {
                env::panic_str("Limit order exceeds maximum leverage")
            }
            _ => {}
        };
    }

    /// Returns the id of the order (new or old if merge happened)
    pub fn add_limit_order(&mut self, mut params: AddLimitOrderParams) -> LimitOrderId {
        self.assert_running();
        if let Some(expiry) = params.expiry {
            assert!(
                expiry > env::block_timestamp_ms(),
                "Limit order already expired"
            );
        }

        let underlying = self.assets.unwrap(&params.underlying_id);

        params.expiry =
            Some(params.expiry.unwrap_or_else(|| {
                env::block_timestamp_ms() + self.max_limit_order_life_sec * 1000
            }));

        let mut limit_order = LimitOrder::new(
            params.clone(),
            if underlying.price > params.price {
                ThresholdType::Below
            } else {
                ThresholdType::Above
            },
        );

        let mut limit_orders = self
            .limit_orders
            .get(&params.underlying_id)
            .unwrap_or_default();

        // Merge orders if they are of the same type at the same price and keep old id. Else
        // generate new id.
        let id = if let Some((existing_id, existing_order)) = limit_orders
            .get_range(
                limit_order.price,
                limit_order.price,
                limit_order.is_long,
                limit_order.threshold,
            )
            .find(|(_, lo)| {
                lo.owner == params.owner
                    && lo.order_type == params.order_type
                    && lo.collateral_id == params.collateral_id
            }) {
            limit_order.collateral_delta += existing_order.collateral_delta;
            limit_order.attached_collateral += existing_order.attached_collateral;
            limit_order.size_delta += existing_order.size_delta;

            self.check_limit_order(&limit_order);

            *existing_id
        } else {
            self.check_limit_order(&limit_order);
            let id = LimitOrderId::new(&limit_order, self.get_limit_order_sequence_number());
            self.insert_limit_order_id(&id, &params);

            id
        };

        limit_orders.insert(id, limit_order.clone());

        self.limit_orders
            .insert(&params.underlying_id, &limit_orders);

        self.validate_order_for_position(&limit_order);

        emit_event(EventType::PlaceLimitOrder(PlaceLimitOrderEvent {
            account_id: params.owner,
            limit_order_id: id.into(),
            collateral_token: limit_order.collateral_id.into_string(),
            underlying_token: limit_order.underlying_id.into_string(),
            order_type: limit_order.order_type.to_string(),
            threshold_type: limit_order.threshold.to_string(),
            collateral_delta_usd: limit_order.collateral_delta.into(),
            attached_collateral_native: limit_order.attached_collateral.into(),
            size_delta_usd: limit_order.size_delta.into(),
            price_usd: limit_order.price.into(),
            expiry: (limit_order.expiry as u128).into(),
            is_long: limit_order.is_long,
        }));

        id
    }

    pub fn limit_order_is_eligible(&self, limit_order: &LimitOrder) -> bool {
        let underlying = self.assets.get(&limit_order.underlying_id).unwrap();
        (matches!(limit_order.threshold, ThresholdType::Above)
            && underlying.price >= limit_order.price)
            || (matches!(limit_order.threshold, ThresholdType::Below)
                && underlying.price <= limit_order.price)
    }

    pub fn get_eligible_orders(&self, asset_id: &AssetId, max: Option<u64>) -> Vec<LimitOrderId> {
        let limit_orders = if let Some(lo) = self.limit_orders.get(asset_id) {
            lo
        } else {
            return vec![];
        };

        let underlying = self.assets.unwrap(asset_id);
        let iter = limit_orders
            .get_range_higher_than_price(underlying.price, true, ThresholdType::Below)
            .chain(limit_orders.get_range_lower_than_price(
                underlying.price,
                true,
                ThresholdType::Above,
            ))
            .chain(limit_orders.get_range_higher_than_price(
                underlying.price,
                false,
                ThresholdType::Below,
            ))
            .chain(limit_orders.get_range_lower_than_price(
                underlying.price,
                false,
                ThresholdType::Above,
            ));
        if let Some(max) = max {
            iter.take(max as usize).map(|e| e.0).cloned().collect()
        } else {
            iter.map(|e| e.0).cloned().collect()
        }
    }

    fn remove_limit_order_id(&mut self, limit_order_id: &LimitOrderId, owner: &AccountId) {
        let mut ids = self.limit_order_ids_map.get(owner).unwrap();
        ids.remove(limit_order_id);
        if !ids.is_empty() {
            self.limit_order_ids_map.insert(owner, &ids);
        } else {
            self.limit_order_ids_map.remove(owner);
        }
    }

    fn insert_limit_order_id(&mut self, id: &LimitOrderId, params: &AddLimitOrderParams) {
        let mut limit_orders_ids = self
            .limit_order_ids_map
            .get(&params.owner)
            .unwrap_or_default();
        limit_orders_ids.insert(*id, params.underlying_id.clone());
        self.limit_order_ids_map
            .insert(&params.owner, &limit_orders_ids);
    }

    #[must_use]
    pub fn execute_limit_order(
        &mut self,
        asset_id: &AssetId,
        limit_order_id: &LimitOrderId,
    ) -> Option<TransferInfo> {
        self.assert_running();
        let mut limit_orders = self
            .limit_orders
            .get(asset_id)
            .expect("No limit orders for this asset exists");
        let limit_order = limit_orders.remove(limit_order_id).unwrap();

        if !self.limit_order_is_eligible(&limit_order) {
            env::panic_str("Position is not ready to be executed");
        }

        if limit_order.expiry < env::block_timestamp_ms() {
            return self.remove_outdated_limit_order(&limit_order.underlying_id, limit_order_id);
        }

        self.limit_orders.insert(asset_id, &limit_orders);
        self.remove_limit_order_id(limit_order_id, &limit_order.owner);

        let res = match limit_order.order_type {
            OrderType::Increase => {
                let (_, transfer_info) = self.increase_position(
                    &limit_order.owner,
                    &limit_order.collateral_id,
                    &limit_order.underlying_id,
                    limit_order.attached_collateral,
                    limit_order.size_delta,
                    limit_order.is_long,
                    Some(*limit_order_id),
                );
                Ok(Some(transfer_info))
            }
            OrderType::Decrease => {
                if let Some(position_id) = self.get_position_for_user(
                    &limit_order.owner,
                    &limit_order.collateral_id,
                    &limit_order.underlying_id,
                    limit_order.is_long,
                ) {
                    let transfer_info = self.decrease_position(
                        position_id,
                        limit_order.collateral_delta,
                        limit_order.size_delta,
                        Some(*limit_order_id),
                        false,
                        None,
                    );
                    Ok(Some(transfer_info))
                } else {
                    Err(())
                }
            }
        };

        match res {
            Ok(r) => {
                emit_event(EventType::RemoveLimitOrder(RemoveLimitOrderEvent {
                    account_id: limit_order.owner,
                    underlying_token: limit_order.underlying_id.into(),
                    limit_order_id: limit_order_id.into(),
                    reason: RemoveOrderReason::Executed,
                    liquidator_id: Some(env::predecessor_account_id()),
                }));
                r
            }
            Err(()) => {
                emit_event(EventType::RemoveLimitOrder(RemoveLimitOrderEvent {
                    account_id: limit_order.owner,
                    underlying_token: limit_order.underlying_id.into(),
                    limit_order_id: limit_order_id.into(),
                    reason: RemoveOrderReason::Invalid,
                    liquidator_id: Some(env::predecessor_account_id()),
                }));
                None
            }
        }
    }

    #[must_use]
    pub fn remove_limit_order(
        &mut self,
        owner_id: &AccountId,
        limit_order_id: &LimitOrderId,
        reason: RemoveOrderReason,
    ) -> Option<TransferInfo> {
        self.assert_running();
        let limit_order_ids = self
            .limit_order_ids_map
            .get(owner_id)
            .expect("You do not have any orders");

        let asset_id = limit_order_ids
            .get(limit_order_id)
            .expect("You do not have any order with this ID");

        let mut limit_orders = self.limit_orders.get(asset_id).unwrap();
        let limit_order = limit_orders.remove(limit_order_id).unwrap();

        self.remove_limit_order_id(limit_order_id, &limit_order.owner);
        self.limit_orders.insert(asset_id, &limit_orders);

        emit_event(EventType::RemoveLimitOrder(RemoveLimitOrderEvent {
            account_id: limit_order.owner.clone(),
            underlying_token: limit_order.underlying_id.into_string(),
            limit_order_id: limit_order_id.into(),
            reason,
            liquidator_id: None,
        }));

        match limit_order.order_type {
            OrderType::Increase => Some(TransferInfo::new(
                &limit_order.owner,
                &limit_order.collateral_id,
                limit_order.attached_collateral,
            )),
            OrderType::Decrease => None,
        }
    }

    #[must_use]
    pub fn remove_outdated_limit_order(
        &mut self,
        asset_id: &AssetId,
        limit_order_id: &LimitOrderId,
    ) -> Option<TransferInfo> {
        self.assert_running();
        let mut limit_orders = self.limit_orders.get(asset_id).unwrap();
        let limit_order = limit_orders.remove(limit_order_id).unwrap();

        if limit_order.expiry > env::block_timestamp_ms() {
            env::panic_str("Limit order has not reached max limit order lifetime");
        }

        self.remove_limit_order_id(limit_order_id, &limit_order.owner);
        self.limit_orders.insert(asset_id, &limit_orders);

        emit_event(EventType::RemoveLimitOrder(RemoveLimitOrderEvent {
            account_id: limit_order.owner.clone(),
            underlying_token: limit_order.underlying_id.into_string(),
            limit_order_id: limit_order_id.into(),
            reason: RemoveOrderReason::Expired,
            liquidator_id: Some(env::predecessor_account_id()),
        }));

        match limit_order.order_type {
            OrderType::Increase => Some(TransferInfo::new(
                &limit_order.owner,
                &limit_order.collateral_id,
                limit_order.attached_collateral,
            )),
            OrderType::Decrease => None,
        }
    }

    fn get_limit_order_sequence_number(&mut self) -> u64 {
        self.limit_order_sequence += 1;
        self.limit_order_sequence
    }

    #[must_use]
    pub fn update_limit_orders(&mut self, account_id: &AccountId, position: &Position) -> Balance {
        let mut amount_out = 0;

        if let Some(user_orders) = self.limit_order_ids_map.get(account_id) {
            let underlying_id = position.underlying_id.clone().into();
            let underlying = self.assets.unwrap(&underlying_id);
            let limit_orders = if let Some(limit_orders) = self.limit_orders.get(&underlying_id) {
                limit_orders
            } else {
                return amount_out;
            };

            for (limit_order_id, asset_id) in user_orders {
                if asset_id != underlying_id {
                    continue;
                }

                let limit_order = limit_orders.get_by_id(&limit_order_id).unwrap();

                if limit_order.collateral_id == position.collateral_id.clone().into()
                    && limit_order.is_long == position.is_long
                {
                    let collateral_usd = self.get_collateral_in_usd(limit_order);
                    let res = self.get_position_new_values(limit_order, position, collateral_usd);
                    if let Ok((new_collateral, new_size)) = res {
                        let new_collateral = self.subtract_possible_losses(
                            position,
                            limit_order,
                            new_size,
                            new_collateral,
                        );

                        if let Some(new_collateral) = new_collateral {
                            if new_size == 0 {
                                continue;
                            } else if new_collateral != 0 {
                                if self
                                    .check_position_size(&underlying, new_size, limit_order.is_long)
                                    .is_ok()
                                    && self.check_leverage(new_size, new_collateral, false).0
                                        == LiquidationStatus::Ok
                                {
                                    continue;
                                }
                            }
                        }
                    };

                    // Remove limit orders that are not valid.
                    let transfer_info = self.remove_limit_order(
                        &position.account_id,
                        &limit_order_id,
                        RemoveOrderReason::Invalid,
                    );

                    if let Some(transfer_info) = transfer_info {
                        amount_out += transfer_info.amount();
                    }
                }
            }
        }
        return amount_out;
    }

    /// Predict position collateral and size values
    /// after executing limit order.
    fn get_position_new_values(
        &self,
        limit_order: &LimitOrder,
        position: &Position,
        collateral_usd: DollarBalance,
    ) -> Result<(DollarBalance, DollarBalance), &str> {
        let current_collateral = position.collateral;
        let current_size = position.size;
        match limit_order.order_type {
            OrderType::Increase => Ok((
                current_collateral + collateral_usd,
                current_size + limit_order.size_delta,
            )),
            OrderType::Decrease => {
                if limit_order.collateral_delta > current_collateral {
                    return Err("Collateral delta must be lower or equal to position collateral");
                }
                if limit_order.size_delta > current_size {
                    return Err("Size delta must be lower or equal to position size");
                }

                Ok((
                    current_collateral - collateral_usd,
                    current_size - limit_order.size_delta,
                ))
            }
        }
    }

    /// Get collateral delta in usd, convert attached token amount in case of
    /// increase position limit order.
    fn get_collateral_in_usd(&self, limit_order: &LimitOrder) -> DollarBalance {
        if matches!(limit_order.order_type, OrderType::Decrease) {
            return limit_order.collateral_delta;
        }

        if limit_order.is_long {
            ratio(
                limit_order.attached_collateral,
                limit_order.price,
                self.assets
                    .unwrap(&limit_order.collateral_id)
                    .denomination(),
            )
        } else {
            limit_order.attached_collateral
        }
    }

    /// Return all user's limit orders
    pub fn get_user_limit_orders(&self, account_id: &AccountId) -> Vec<LimitOrderView> {
        let mut user_orders = Vec::new();

        if let Some(order_ids) = self.limit_order_ids_map.get(account_id) {
            for (limit_order_id, asset_id) in order_ids {
                let limit_orders = self.limit_orders.get(&asset_id).unwrap();
                let limit_order = limit_orders.get_by_id(&limit_order_id).unwrap();
                user_orders.push(LimitOrderView::new(limit_order, &limit_order_id));
            }
        }

        user_orders
    }

    /// Check that there is only one limit order of certain type for current position.
    /// Panic if position already has such order.
    fn validate_order_for_position(&self, limit_order: &LimitOrder) {
        let user_orders = self.get_user_limit_orders(&limit_order.owner);
        let orders_amount = user_orders
            .iter()
            .filter(|order| {
                order.collateral_id == limit_order.collateral_id.into_string()
                    && order.underlying_id == limit_order.underlying_id.into_string()
                    && order.is_long == limit_order.is_long
                    && order.order_type == limit_order.order_type
                    && order.threshold == limit_order.threshold
            })
            .count();

        assert!(
            orders_amount == 1,
            "Position already has a limit order of such type"
        );
    }

    /// Check if there are losses at the time limit order has to be executed.
    /// If they are present, subtract them from remaining collateral.
    fn subtract_possible_losses(
        &self,
        position: &Position,
        limit_order: &LimitOrder,
        new_size: DollarBalance,
        new_collateral: DollarBalance,
    ) -> Option<u128> {
        let (has_profit, delta) = get_delta(
            position.average_price,
            limit_order.price,
            new_size,
            position.is_long,
        );

        if !has_profit {
            new_collateral.checked_sub(delta)
        } else {
            Some(new_collateral)
        }
    }
}

/// A data structure to store limit orders They are
/// stored from long to short, from below threshold
/// to above threshold, from lowest price to highest
/// price. This allows for very fast and precise
/// queries and batch deletes.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LimitOrders(BTreeMap<LimitOrderId, LimitOrder>);

impl LimitOrders {
    pub fn new() -> Self {
        LimitOrders(BTreeMap::new())
    }

    pub fn insert(&mut self, id: LimitOrderId, limit_order: LimitOrder) {
        self.0.insert(id, limit_order);
    }

    pub fn get_by_id(&self, limit_order_id: &LimitOrderId) -> Option<&LimitOrder> {
        self.0.get(limit_order_id)
    }

    pub fn get_range(
        &mut self,
        price_min: u128,
        price_max: u128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> Range<LimitOrderId, LimitOrder> {
        let min_id = LimitOrderId::get_min_id_from_price(price_min, is_long, threshold);
        let max_id = LimitOrderId::get_max_id_from_price(price_max, is_long, threshold);
        self.0.range((Included(min_id), Included(max_id)))
    }

    pub fn get_range_higher_than_price(
        &self,
        price: u128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> Range<LimitOrderId, LimitOrder> {
        let min_id = LimitOrderId::get_min_id_from_price(price, is_long, threshold);
        let max_id = LimitOrderId::get_max_id(is_long, threshold);
        self.0.range((Included(min_id), Included(max_id)))
    }

    pub fn get_range_lower_than_price(
        &self,
        price: u128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> Range<LimitOrderId, LimitOrder> {
        let min_id = LimitOrderId::get_min_id(is_long, threshold);
        let max_id = LimitOrderId::get_max_id_from_price(price, is_long, threshold);
        self.0.range((Included(min_id), Included(max_id)))
    }

    pub fn remove(&mut self, id: &LimitOrderId) -> Option<LimitOrder> {
        self.0.remove(id)
    }

    pub fn to_vec(&self) -> Vec<LimitOrder> {
        self.0.iter().map(|e| e.1).cloned().collect()
    }

    pub fn to_entries_vec(&self) -> Vec<(LimitOrderId, LimitOrder)> {
        self.0.iter().map(|e| (*e.0, e.1.clone())).collect()
    }
}

impl Default for LimitOrders {
    fn default() -> Self {
        Self::new()
    }
}
