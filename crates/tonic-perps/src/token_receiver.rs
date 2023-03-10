use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{json_types::U128, PromiseOrValue};
use tonic_perps_sdk::prelude::{emit_event, EventType, TokenDepositWithdrawEvent};

use crate::{
    env, near_bindgen, AccountId, Action, AddLimitOrderParams, AssetId, IncreasePositionRequest,
    OrderType, VContract, VContractExt,
};

#[near_bindgen]
impl FungibleTokenReceiver for VContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.contract_mut().assert_running();
        let token_id = env::predecessor_account_id();
        let asset_id = AssetId::Ft(token_id);
        let action = serde_json::from_str::<Action>(&msg).expect("invalid message");

        match action {
            Action::Swap(params) => {
                if let Some(referrer_id) = params.referrer_id {
                    self.set_user_referral_code(referrer_id);
                }
                let contract = self.contract_mut();
                contract.swap_and_send(
                    &asset_id,
                    &params.output_token_id.into(),
                    amount.0,
                    params.min_out.map(|x| x.0),
                    &sender_id,
                );
            }
            Action::MintLp(params) => {
                if let Some(referrer_id) = params.referrer_id {
                    self.set_user_referral_code(referrer_id);
                }
                let contract = self.contract_mut();
                contract.mint_lp_token(
                    &sender_id,
                    &asset_id,
                    amount.0,
                    params.min_out.map(|a| a.0),
                );
            }
            Action::IncreasePosition(params) => {
                let IncreasePositionRequest {
                    is_long,
                    underlying_id,
                    size_delta,
                    ..
                } = params;
                if let Some(referrer_id) = params.referrer_id {
                    self.set_user_referral_code(referrer_id);
                }
                let contract = self.contract_mut();
                contract.increase_position(
                    &sender_id,
                    &asset_id,
                    &underlying_id.into(),
                    amount.0,
                    size_delta.0,
                    is_long,
                    None,
                );
            }
            Action::PlaceLimitOrder(params) => {
                let contract = self.contract_mut();
                contract.assert_limit_order_state(matches!(params.order_type, OrderType::Increase));

                assert!(
                    matches!(params.order_type, OrderType::Increase)
                        && params.collateral_delta.is_none(),
                    "Collateral field is only applicable to sell orders"
                );
                contract.add_limit_order(AddLimitOrderParams {
                    owner: sender_id.clone(),
                    collateral_id: asset_id,
                    underlying_id: AssetId::from(params.underlying_id),
                    collateral_delta_usd: 0,
                    attached_collateral_native: amount.0,
                    size: params.size_delta.0,
                    price: params.price.0,
                    is_long: params.is_long,
                    order_type: params.order_type,
                    expiry: params.expiry.map(|e| e.0),
                });
            }
        };

        emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
            amount_native: amount,
            deposit: true,
            method: "ft_on_transfer".to_string(),
            receiver_id: env::current_account_id(),
            account_id: sender_id,
            asset_id: env::predecessor_account_id().to_string(),
        }));

        PromiseOrValue::Value(U128(0))
    }
}
