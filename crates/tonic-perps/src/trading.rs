use near_sdk::{env, json_types::U128, near_bindgen, AccountId, Balance, Gas, PromiseOrValue};
use tonic_perps_sdk::prelude::{FeeType, TokenDepositWithdrawEvent};

const GAS_SURPLUS: u64 = 7;

use crate::{
    convert_assets, emit_event, AssetId, Contract, EventType, SwapEvent, SwapState, TransferInfo,
    VContract, VContractExt, TGAS_FOR_FT_TRANSFER,
};

impl Contract {
    /// Perform a swap. Return the amount of output token.
    ///
    /// It's the caller's responsibility to send the token back to
    /// user.
    ///
    /// Panics if slippage tolerance is exceeded.
    #[must_use]
    pub fn swap(
        &mut self,
        token_in: &AssetId,
        token_out: &AssetId,
        amount_in: Balance,
        min_amount_out: Option<Balance>,
    ) -> Balance {
        self.assert_swap_enabled();
        self.validate_asset_price(token_in);
        self.validate_asset_price(token_out);
        let mut asset_in = self.assets.unwrap(token_in);
        let mut asset_out = self.assets.unwrap(token_out);
        let account_id = env::signer_account_id();

        if token_in == token_out {
            env::panic_str("Swap tokens should be different");
        }

        if self.owner_id != env::signer_account_id() {
            assert!(
                asset_in.state.swap.check(SwapState::Enabled)
                    || asset_in.state.swap.check(SwapState::InOnly)
            );
            assert!(
                asset_out.state.swap.check(SwapState::Enabled)
                    || asset_out.state.swap.check(SwapState::OutOnly)
            );
        }

        let amount_out = {
            convert_assets(
                amount_in,
                asset_in.min_price(),
                asset_out.denomination(),
                asset_out.max_price(),
                asset_in.denomination(),
            )
        };

        if let Some(min_amount_out) = min_amount_out {
            assert!(amount_out >= min_amount_out, "Exceeded slippage tolerance");
        }

        assert!(
            amount_out <= asset_out.available_liquidity(),
            "Not enough liquidity to perform swap {} {}",
            amount_out,
            asset_out.available_liquidity()
        );

        let swap_fee_bps = self.get_swap_fee_bps(token_in, token_out, amount_in, amount_out);
        let (after_fee_amount, fees) = self.withhold_fees(amount_out, swap_fee_bps);

        // do this in a weird way due to borrow checker
        asset_in.add_liquidity(amount_in, &account_id);
        self.update_cumulative_funding_rate(&mut asset_in);

        asset_out.remove_liquidity(amount_out, &account_id);
        asset_out.add_fees(fees, FeeType::Swap, &account_id);
        asset_out.check_available_liquidity();
        self.update_cumulative_funding_rate(&mut asset_out);

        emit_event(EventType::Swap(SwapEvent {
            account_id: env::signer_account_id(),
            token_in: token_in.into_string(),
            token_out: token_out.into_string(),
            amount_in_native: amount_in.into(),
            amount_out_native: amount_out.into(),
            amount_in_usd: asset_in.to_max_usd_price(amount_in).into(),
            amount_out_usd: asset_out.to_max_usd_price(amount_out).into(),
            fees_native: fees.into(),
            fees_usd: asset_out.to_max_usd_price(fees).into(),
            fee_bps: swap_fee_bps.into(),
            referral_code: self.user_referral_code.get(&env::signer_account_id()),
        }));

        self.set_asset(token_in, asset_in);
        self.set_asset(token_out, asset_out);

        after_fee_amount
    }

    /// Perform a swap. Return the amount of output token.
    ///
    /// It's the caller's responsibility to send the token back to
    /// user.
    ///
    /// Panics if slippage tolerance is exceeded.
    pub fn swap_and_send(
        &mut self,
        token_in: &AssetId,
        token_out: &AssetId,
        amount_in: Balance,
        min_amount_out: Option<Balance>,
        receiver_id: &AccountId,
    ) -> PromiseOrValue<()> {
        let amount_out = self.swap(token_in, token_out, amount_in, min_amount_out);
        let transfer_info = TransferInfo::new(receiver_id, token_out, amount_out);

        self.internal_send(transfer_info, "swap_and_send")
    }
}

#[near_bindgen]
impl VContract {
    #[payable]
    pub fn swap_near(
        &mut self,
        output_token_id: String,
        min_out: Option<U128>,
        referrer_id: Option<String>,
    ) {
        self.contract_mut().assert_running();
        if let Some(referrer_id) = referrer_id {
            self.set_user_referral_code(referrer_id);
        }
        let contract = self.contract_mut();
        let sender_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        contract.swap_and_send(
            &AssetId::NEAR,
            &output_token_id.into(),
            amount,
            min_out.map(Into::into),
            &sender_id,
        );

        emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
            amount_native: env::attached_deposit().into(),
            deposit: true,
            method: "swap_near".to_string(),
            receiver_id: env::current_account_id(),
            account_id: env::predecessor_account_id(),
            asset_id: AssetId::NEAR.into_string(),
        }));
    }

    #[payable]
    pub fn withdraw_fees(&mut self, asset_ids: Option<Vec<String>>) -> U128 {
        let receiver_id = env::predecessor_account_id();
        let contract = self.contract_mut();
        contract.assert_admin();

        let asset_ids: Vec<AssetId> = if let Some(asset_ids) = asset_ids {
            asset_ids.iter().map(|asset| asset.clone().into()).collect()
        } else {
            contract.assets.0.keys().cloned().collect()
        };

        let mut fees_usd = 0;
        for asset_id in asset_ids {
            let asset_id = AssetId::from(asset_id);
            let mut asset = contract.assets.unwrap(&asset_id);
            let fee_native = asset.accumulated_fees;

            contract.update_cumulative_funding_rate(&mut asset);

            asset.remove_fees(fee_native, FeeType::WithrawFee, &receiver_id);
            fees_usd += asset.to_min_usd_price(fee_native);

            contract.set_asset(&asset_id, asset);

            let transfer_info = TransferInfo::new(&receiver_id, &asset_id, fee_native);
            contract.internal_send(transfer_info, "withdraw_fee");

            if check_gas_leftover() {
                break;
            }
        }

        fees_usd.into()
    }
}

fn check_gas_leftover() -> bool {
    env::prepaid_gas() - env::used_gas() < Gas::ONE_TERA * (TGAS_FOR_FT_TRANSFER + GAS_SURPLUS)
}
