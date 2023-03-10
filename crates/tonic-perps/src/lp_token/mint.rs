use near_sdk::json_types::U128;
use tonic_perps_sdk::prelude::FeeType;

use crate::{
    convert_assets, emit_event, env, near_bindgen, ratio, AccountId, AssetId, Balance, Contract,
    DollarBalance, EventType, LpSupportState, MintBurnDirection, MintBurnLpEvent, TransferInfo,
    VContract, VContractExt, DOLLAR_DENOMINATION, LP_TOKEN_DENOMINATION,
};

impl Contract {
    /// Mint LP tokens. Increase liquidity in the pool.
    pub fn mint_lp_token(
        &mut self,
        account_id: &AccountId,
        asset_id: &AssetId,
        deposit: Balance,
        min_out: Option<Balance>,
    ) -> Balance {
        self.validate_asset_price(asset_id);
        assert!(deposit > 0, "Deposit amount should be positive");

        let total_aum = self.get_total_aum(); // XXX: io
        let prev_supply = self.lp_token.total_supply;

        let mut asset = self.assets.unwrap(asset_id);
        assert!(asset.state.lp_support.check(LpSupportState::Enabled));

        let mint_fee_bps = self.get_mint_fee_bps(&asset, deposit);
        let (after_fee_amount, fees) = self.withhold_fees(deposit, mint_fee_bps);

        asset.add_fees(fees, FeeType::Mint, account_id);
        asset.add_liquidity(after_fee_amount, account_id);
        let fees_usd = asset.dollar_value_of(fees);
        self.update_cumulative_funding_rate(&mut asset);

        let mint_amount = get_lp_mint_amount(
            total_aum,
            asset.price,
            asset.denomination(),
            after_fee_amount,
            prev_supply,
        );
        if let Some(min_out) = min_out {
            assert!(mint_amount >= min_out, "exceeded slippage tolerance");
        }

        asset.register_deposit(mint_amount);

        self.lp_token.internal_deposit(account_id, mint_amount);
        self.set_asset(asset_id, asset);

        emit_event(EventType::MintBurnLp(MintBurnLpEvent {
            direction: MintBurnDirection::Mint,
            account_id: account_id.clone(),
            token_in: asset_id.into_string(),
            amount_in: deposit.into(),
            token_out: env::current_account_id().to_string(),
            amount_out: mint_amount.into(),
            fees: fees.into(),
            fees_usd: fees_usd.into(),
            fees_bps: mint_fee_bps.into(),
            lp_price_usd: self.lp_price().into(),
        }));
        mint_amount
    }

    /// Redeem some amount of the LP token for an asset. Decrease liquidity in
    /// the pool and burn the LP token.
    ///
    /// Return amount of asset redeemed. It's the caller's responsibility to
    /// send this amount from the treasury back to the user.
    #[must_use]
    pub fn burn_lp_token(
        &mut self,
        account_id: &AccountId,
        burn_amount: Balance,
        output_asset_id: &AssetId,
        min_out: Option<Balance>,
    ) -> Balance {
        self.validate_asset_price(output_asset_id);
        let mut asset = self.assets.unwrap(output_asset_id);
        assert!(!asset.state.lp_support.check(LpSupportState::Disabled));

        let prev_supply = self.lp_token.total_supply;
        let prev_price = self.lp_price();
        self.lp_token.internal_withdraw(account_id, burn_amount);

        let redemption_amount = get_lp_redemption_amount(
            self.get_total_aum(),
            asset.price,
            asset.denomination(),
            burn_amount,
            prev_supply,
        );
        if let Some(min_out) = min_out {
            assert!(redemption_amount >= min_out, "exceeded slippage tolerance");
        }

        assert!(
            redemption_amount <= asset.available_liquidity(),
            "Not enough liquidity to burn tokens {} {}",
            redemption_amount,
            asset.available_liquidity()
        );

        asset.register_withdrawal(redemption_amount);

        let burn_fee_bps = self.get_burn_fee_bps(&asset, burn_amount);
        let (redemption_amount_after_fees, fees) =
            self.withhold_fees(redemption_amount, burn_fee_bps);

        asset.remove_liquidity(redemption_amount, account_id);
        asset.add_fees(fees, FeeType::Burn, account_id);
        asset.check_available_liquidity();

        self.update_cumulative_funding_rate(&mut asset);
        let fees_usd = asset.dollar_value_of(fees);
        self.set_asset(output_asset_id, asset);

        emit_event(EventType::MintBurnLp(MintBurnLpEvent {
            direction: MintBurnDirection::Burn,
            account_id: account_id.clone(),
            amount_in: burn_amount.into(),
            token_in: env::current_account_id().to_string(),
            token_out: output_asset_id.into_string(),
            amount_out: redemption_amount.into(),
            fees: fees.into(),
            fees_usd: fees_usd.into(),
            fees_bps: burn_fee_bps.into(),
            lp_price_usd: prev_price.into(),
        }));

        redemption_amount_after_fees
    }

    pub fn lp_price(&self) -> DollarBalance {
        assert!(
            self.lp_token.total_supply > 0,
            "Price as unavailable due to lp supply absence"
        );
        ratio(
            self.get_total_aum(),
            LP_TOKEN_DENOMINATION,
            self.lp_token.total_supply,
        )
    }
}

#[near_bindgen]
impl VContract {
    #[payable]
    pub fn mint_lp_near(&mut self, min_out: Option<U128>, referrer_id: Option<String>) -> U128 {
        self.contract_mut().assert_running();
        if let Some(referrer_id) = referrer_id {
            self.set_user_referral_code(referrer_id);
        }
        let contract = self.contract_mut();
        contract
            .mint_lp_token(
                &env::predecessor_account_id(),
                &AssetId::NEAR,
                env::attached_deposit(),
                min_out.map(|a| a.0),
            )
            .into()
    }

    #[payable]
    pub fn burn_lp_token(
        &mut self,
        amount: U128,
        output_token_id: String,
        min_out: Option<U128>,
        referrer_id: Option<String>,
    ) -> U128 {
        self.contract_mut().assert_running();
        if let Some(referrer_id) = referrer_id {
            self.set_user_referral_code(referrer_id);
        }
        let contract = self.contract_mut();
        let asset_id = &AssetId::from(output_token_id);
        let account_id = env::predecessor_account_id();
        let balance = contract.burn_lp_token(&account_id, amount.0, asset_id, min_out.map(|a| a.0));

        let transfer_info = TransferInfo::new(&account_id, asset_id, balance);
        contract.internal_send(transfer_info, "burn_lp_token");

        balance.into()
    }
}

/// Get number of shares in the pool to issue given a deposit in terms of USD.
pub fn get_lp_mint_amount(
    total_aum: DollarBalance,
    asset_price: DollarBalance,
    asset_denomination: Balance,
    after_fee_amount: DollarBalance,
    prev_supply: Balance,
) -> Balance {
    if prev_supply == 0 {
        // pools are all empty, mint one share per dollar
        let deposit_value = ratio(after_fee_amount, asset_price, asset_denomination);
        ratio(LP_TOKEN_DENOMINATION, deposit_value, DOLLAR_DENOMINATION)
    } else {
        convert_assets(
            after_fee_amount,
            prev_supply,
            asset_price,
            total_aum,
            asset_denomination,
        )
    }
}

/// Get amount of underlying to return when burning LP tokens
///
/// Formula is:
///
/// ```md
/// dollar_value_out = percentage_of_shares * total_aum
/// tokens_out = dollar_value_out / price_per_token
/// ```
///
/// In code,
/// ```md
/// asset_denomination * total_aum * (burn_amount / total_supply) / price
/// ```
pub fn get_lp_redemption_amount(
    total_aum: DollarBalance,
    asset_price: DollarBalance,
    asset_denomination: Balance,
    burn_amount: Balance,
    total_supply: Balance,
) -> Balance {
    let amount_out = convert_assets(
        burn_amount,
        total_aum,
        asset_denomination,
        total_supply,
        asset_price,
    );

    amount_out
}
