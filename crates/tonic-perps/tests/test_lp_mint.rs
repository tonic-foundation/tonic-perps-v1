mod common;

use common::*;

use near_contract_standards::{
    fungible_token::core::FungibleTokenCore, fungible_token::receiver::FungibleTokenReceiver,
};

#[test]
#[should_panic(expected = "Price as unavailable due to lp supply absence")]
fn test_zero_lp_supply() {
    let (_, vcontract) = setup();
    vcontract.get_lp_price();
}

#[test]
fn lp_mint_amounts() {
    {
        let total_aum = dollars(100);
        let deposit_value = dollars(10);
        let prev_supply = lp_tokens(100.);

        assert_eq!(
            get_lp_mint_amount(
                total_aum,
                deposit_value,
                dollars(1),
                DOLLAR_DENOMINATION,
                prev_supply
            ),
            lp_tokens(10.),
            "100 case failed"
        )
    }

    {
        let total_aum = 0;
        let deposit_value = dollars(10);
        let prev_supply = 0;

        assert_eq!(
            get_lp_mint_amount(
                total_aum,
                deposit_value,
                dollars(1),
                DOLLAR_DENOMINATION,
                prev_supply
            ),
            lp_tokens(10.),
            "0 case failed"
        )
    }
}

#[test]
fn lp_burn_amounts() {
    let asset_denomination = near(1);

    let total_aum = dollars(1000);
    let asset_price = dollars(25);
    let prev_supply = lp_tokens(100.);

    // redeem 10% of shares, pool is 25% of aum
    // 1000 aum with 100 shares, pool value 250. burn 10 shares for pool
    // tokens, ie, (10% of 1000) / price = 100 / 25 = 4 pool tokens
    let burn_amount = lp_tokens(10.);
    assert_eq!(
        get_lp_redemption_amount(
            total_aum,
            asset_price,
            asset_denomination,
            burn_amount,
            prev_supply
        ),
        near(4),
    );

    // redeem 25% of shares, pool is 25% of aum
    // should return the total pool size
    let burn_amount = lp_tokens(25.);
    assert_eq!(
        get_lp_redemption_amount(
            total_aum,
            asset_price,
            asset_denomination,
            burn_amount,
            prev_supply
        ),
        near(10),
    );
}

#[test]
fn lp_burn_amounts_stable() {
    let asset_denomination = dollars(1);

    let total_aum = dollars(200000);
    let asset_price = dollars(1);
    let prev_supply = lp_tokens(200000.0);

    // redeem 0.005% of shares, pool is 0.005% of aum,
    // 10 lp_tokens * $200k aum / 200k total_supply = $10 - value of output token
    // $10 * 10^6 USDT decimals / 10^6 USDT price = 10 USDT
    let burn_amount = lp_tokens(10.);
    assert_eq!(
        get_lp_redemption_amount(
            total_aum,
            asset_price,
            asset_denomination,
            burn_amount,
            prev_supply
        ),
        dollars(10),
    );

    let total_aum = dollars(200000);
    let asset_price = dollars(1);
    let prev_supply = lp_tokens(200000.0);

    // 1 * 10^11 lp_tokens * $200k aum / 200k total_supply = 0.000001 = $0 - value of output token
    // $0 * 10^6 USDT decimals / 10^6 USDT price = 0 USDT
    // Value of lp tokens is too low to be converted in positive amount of USDT
    let burn_amount = 10u128.pow(11);
    assert_eq!(
        get_lp_redemption_amount(
            total_aum,
            asset_price,
            asset_denomination,
            burn_amount,
            prev_supply
        ),
        dollars(0),
    );
}

#[test]
fn test_mint_lp_token() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;

    // LP token # = dollar amounts when there's nothing in the pool
    assert_eq!(balance, lp_tokens(500.0), "Incorrect LP token mint amount");

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Bob),
        dollars(5).into(),
        serde_json::to_string(&Action::MintLp(MintLpParams {
            min_out: None,
            referrer_id: None,
        }))
        .unwrap(),
    );
    let balance = vcontract.ft_balance_of(get_account(Bob)).0;
    assert_eq!(balance, lp_tokens(5.0), "Incorrect LP token mint amount");

    assert_eq!(
        vcontract.ft_total_supply().0,
        lp_tokens(505.),
        "Incorrect LP token supply"
    );
}

#[test]
fn test_mint_lp_token_low_decimals() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    // Minimum deposit amount - $0.000001 to create first lp token
    set_deposit(&mut context, 2 * 10u128.pow(17));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;

    // LP token # = dollar amounts when there's nothing in the pool
    assert_eq!(balance, 1_000_000_000_000, "Incorrect LP token mint amount"); // 0.000001 LP tokens

    set_predecessor(&mut context, Bob);
    set_deposit(&mut context, 5 * 10u128.pow(16)); // 0.00000005 NEAR / $0
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Bob)).0;
    assert_eq!(balance, 250_000_000_000, "Incorrect LP token mint amount"); // 0.00000025 LP tokens
}

#[test]
#[should_panic(expected = "Price as unavailable due to lp supply absence")]
fn test_mint_low_decimals_first_deposit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    // Minimum deposit amount - $0.000001 to create first lp token
    // or 0.0000002 NEAR, deposit less
    set_deposit(&mut context, 2 * 10u128.pow(17) - 1);
    vcontract.mint_lp_near(None, None);
}

#[test]
fn test_burn_lp_tokens_low_decimals() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(500.0), "Incorrect LP token mint amount");

    // Burn lp tokens to get NEAR
    let amount_out = vcontract.burn_lp_token(U128(2 * 10u128.pow(5)), near_id(), None, None);
    // amount out = 0.00000000000004 NEAR = $0
    assert_eq!(amount_out.0, 40_000_000_000);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, 499999999999999800000, "Incorrect LP tokens burned");
}

#[test]
#[should_panic(expected = "Asset price is too stale")]
fn test_mint_lp_token_stale_price() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // Default price staleness maximum is 90sec, add 91sec
    context.block_timestamp(
        context.context.block_timestamp + std::time::Duration::from_secs(91).as_nanos() as u64,
    );
    testing_env!(context.build());

    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);
}

#[test]
#[should_panic(expected = "Price should be greater than 0")]
fn test_mint_lp_token_zero_price() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(0));

    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);
}

#[test]
#[should_panic(expected = "Asset price is too stale")]
fn test_burn_lp_token_stale_price() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    // Default price staleness maximum is 90sec, add 91sec
    context.block_timestamp(
        context.context.block_timestamp + std::time::Duration::from_secs(91).as_nanos() as u64,
    );
    testing_env!(context.build());

    vcontract.burn_lp_token(U128(lp_tokens(500.)), near_id(), None, None);
}

#[test]
#[should_panic(expected = "Exceed max possible pool amount for this asset")]
fn test_exceed_max_lp_tokens() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_max_pool_amount(near_id(), U128(near(100)));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(101));
    // MAX pool amount for NEAR asset - 100 tokens, attach 101 tokens
    vcontract.mint_lp_near(None, None);
}

#[test]
fn test_mint_fee() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 10,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(20));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;

    assert_eq!(balance, lp_tokens(99.9), "No fee deducted from LP");

    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(
        near_asset.accumulated_fees.0,
        near(20) / 1000, // 0.1% of 20 NEAR
        "Did not accumulate fees after mint"
    );

    assert_eq!(
        near_asset.pool_amount.0,
        near(20) - near(20) / 1000, // 20 NEAR - 0.1% of 20
        "Incorrect pool balance after mint"
    );
}

#[test]
#[should_panic]
fn test_burn_only_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(100));
    set_predecessor(&mut context, Admin);
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Disabled,
            lp_support: LpSupportState::BurnOnly,
            swap: SwapState::Disabled,
        },
    );

    set_predecessor(&mut context, Alice);
    vcontract.mint_lp_near(None, None);
}

#[test]
#[should_panic(expected = "Contract is temporary paused")]
fn test_mint_paused_contract() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(100));
    set_predecessor(&mut context, Admin);
    vcontract.set_state(ContractState::Paused);

    set_predecessor(&mut context, Alice);
    vcontract.mint_lp_near(None, None);
}

#[test]
#[should_panic]
fn test_disabled_lp_support_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(100));
    set_predecessor(&mut context, Admin);
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Disabled,
            swap: SwapState::Enabled,
        },
    );

    set_predecessor(&mut context, Alice);
    vcontract.mint_lp_near(None, None);
}

#[test]
fn test_burn() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_withdrawal_limits_settings(near_id(), Some(0), Some(U128(0)));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(20));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(100.0), "Incorrect LP tokens minted");

    set_predecessor(&mut context, Admin);
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 10,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    set_predecessor(&mut context, Alice);
    vcontract.burn_lp_token(U128(lp_tokens(50.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(50.), "Incorrect LP tokens burned");

    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(
        near_asset.accumulated_fees.0,
        near(10) / 1000, // 0.1% of 10 NEAR
        "Did not accumulate fees after burn"
    );

    assert_eq!(
        near_asset.pool_amount.0,
        near(10),
        "Incorrect pool balance after burn"
    );
}

#[test]
fn test_burn_in_burn_only_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_withdrawal_limits_settings(near_id(), Some(0), Some(U128(0)));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(20));
    vcontract.mint_lp_near(None, None);

    set_predecessor(&mut context, Admin);
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Disabled,
            lp_support: LpSupportState::BurnOnly,
            swap: SwapState::Disabled,
        },
    );

    set_predecessor(&mut context, Alice);
    vcontract.burn_lp_token(U128(lp_tokens(50.)), near_id(), None, None);
}

#[test]
#[should_panic]
fn test_burn_lp_token_not_enough_liquidity() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // add liquidity to NEAR
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let near_asset = vcontract.get_asset_info(near_id());

    // Initial pool 100 NEAR + collateral 5 NEAR.
    assert_eq!(near_asset.pool_amount.0, near(105));
    // Pool amount 105 NEAR - 20 NEAR of reserve amount.
    assert_eq!(near_asset.available_liquidity.0, near(85));

    let lp_price = vcontract.get_lp_price();
    let dollar_value = ratio(
        near_asset.available_liquidity.0,
        dollars(5),
        NEAR_DENOMINATION,
    ) + 1;
    let lp_amount = ratio(dollar_value, LP_TOKEN_DENOMINATION, lp_price.0);

    // Burn lp token amount that exceeds available liquidity.
    vcontract.burn_lp_token(U128(lp_amount), near_id(), None, None);
}

#[test]
#[should_panic(expected = "Deposit amount should be positive")]
fn test_mint_zero_deposit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(0));
    vcontract.mint_lp_near(None, None);
}
