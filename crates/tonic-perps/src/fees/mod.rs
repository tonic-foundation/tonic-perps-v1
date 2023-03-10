use crate::{ratio, Asset, AssetId, Balance, Contract, DollarBalance, BN, FUNDING_RATE_PRECISION};

pub const ONE_PERCENT_BPS: u128 = 100;
pub const BPS_DIVISOR: u128 = 10_000;

pub fn get_funding_fee(
    position_size: DollarBalance,
    entry_funding_rate: u128,
    cumulative_funding_rate: u128,
) -> DollarBalance {
    if position_size == 0 || cumulative_funding_rate == 0 {
        return 0;
    }
    ratio(
        position_size,
        cumulative_funding_rate - entry_funding_rate,
        FUNDING_RATE_PRECISION,
    )
}

impl Contract {
    pub fn get_swap_fee_bps(
        &self,
        asset_in: &AssetId,
        asset_out: &AssetId,
        amount_in: Balance,
        amount_out: Balance,
    ) -> u16 {
        let asset_in = self.assets.unwrap(asset_in);
        let asset_out = self.assets.unwrap(asset_out);
        let is_stableswap = asset_in.stable && asset_out.stable;

        if is_stableswap {
            return self.fee_parameters.stable_swap_fee_bps;
        }

        let total_usd = self.get_total_value();
        let base_bps = self.fee_parameters.swap_fee_bps;
        let fee_bps_in = self.get_fee_bps(
            &asset_in,
            amount_in,
            true,
            base_bps,
            self.fee_parameters.tax_bps,
            total_usd,
        );
        let fee_bps_out = self.get_fee_bps(
            &asset_out,
            amount_out,
            false,
            base_bps,
            self.fee_parameters.tax_bps,
            total_usd,
        );

        fee_bps_in.max(fee_bps_out)
    }

    fn get_total_value(&self) -> DollarBalance {
        let total_usd: DollarBalance = self
            .assets
            .0
            .values()
            .map(|asset| asset.dollar_value_of(asset.pool_balance))
            .sum();

        total_usd
    }

    pub fn get_mint_fee_bps(&self, asset: &Asset, amount: Balance) -> u16 {
        let total_usd = self.get_total_value();
        self.get_fee_bps(
            asset,
            amount,
            true,
            self.fee_parameters.mint_burn_fee_bps,
            self.fee_parameters.tax_bps,
            total_usd,
        )
    }

    pub fn get_burn_fee_bps(&self, asset: &Asset, amount: Balance) -> u16 {
        let total_usd = self.get_total_value();
        self.get_fee_bps(
            asset,
            amount,
            false,
            self.fee_parameters.mint_burn_fee_bps,
            self.fee_parameters.tax_bps,
            total_usd,
        )
    }

    pub fn get_fee_bps(
        &self,
        asset: &Asset,
        amount: Balance,
        increase: bool,
        fee_bps: u16,
        tax_bps: u16,
        total_usd_value: DollarBalance,
    ) -> u16 {
        if !self.dynamic_swap_fees || self.total_weights == 0 || total_usd_value == 0 {
            return fee_bps;
        }

        let target_dollars = ratio(total_usd_value, asset.token_weight, self.total_weights);

        let current_dollars = asset.dollar_value_of(asset.pool_balance);
        let dollar_delta = asset.dollar_value_of(amount);

        let next_amount = if increase {
            current_dollars + dollar_delta
        } else {
            current_dollars.saturating_sub(dollar_delta)
        };

        let initial_diff = current_dollars.abs_diff(target_dollars);
        let next_diff = next_amount.abs_diff(target_dollars);

        if next_diff < initial_diff {
            let rebate_bps = BN(initial_diff.into())
                .mul(tax_bps.into())
                .div(target_dollars)
                .as_u64() as u16;
            return std::cmp::max(fee_bps.saturating_sub(rebate_bps), 1);
        }
        let average_diff = std::cmp::min((initial_diff + next_diff) / 2, target_dollars);
        let final_tax_bps = BN(average_diff.into())
            .mul(tax_bps.into())
            .div(target_dollars)
            .as_u64();

        fee_bps + (final_tax_bps as u16)
    }

    /// Returns fee in USD for a position
    pub fn get_position_fee(
        &self,
        size_delta: DollarBalance,
        asset: &Asset,
        is_long: bool,
    ) -> DollarBalance {
        let skew = if self.dynamic_position_fees
            && asset.global_short_size + asset.global_long_size != 0
        {
            // Get the percentage of current position type out of total
            let skew = if is_long {
                ratio(
                    asset.global_long_size,
                    BPS_DIVISOR,
                    asset.global_short_size + asset.global_long_size,
                )
            } else {
                ratio(
                    asset.global_short_size,
                    BPS_DIVISOR,
                    asset.global_short_size + asset.global_long_size,
                )
            };

            // Double (if current position type represents 50% of total open interest,
            // the fee should be 100% <=> unchanged)
            let skew = skew * 2;

            // Cap maximum impact of skew
            skew.clamp(5000, 20000)
        } else {
            BPS_DIVISOR
        };

        let margin_fee = ratio(size_delta, self.fee_parameters.margin_fee_bps, BPS_DIVISOR);

        ratio(margin_fee, skew, BPS_DIVISOR)
    }

    pub fn withhold_fees(&self, amount: Balance, fee_bps: u16) -> (u128, u128) {
        let fees = ratio(amount, fee_bps, BPS_DIVISOR);
        (amount - fees, fees)
    }
}
