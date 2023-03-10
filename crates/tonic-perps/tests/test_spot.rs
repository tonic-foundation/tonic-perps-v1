mod common;

use common::*;

#[test]
fn test_spot() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);

    assert_eq!(
        vcontract.get_asset_info(near_id()).available_liquidity.0,
        near(90),
    );
    assert_eq!(
        vcontract.get_asset_info(usdc_id()).available_liquidity.0,
        dollars(50)
    );
}

#[test]
fn test_swap_near() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(500));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(20));

    let near_before = vcontract.get_asset_info(near_id());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    vcontract.swap_near(usdc_id(), None, None);

    let near_after = vcontract.get_asset_info(near_id());
    let usdc_after = vcontract.get_asset_info(usdc_id());
    assert_eq!(
        near_after.pool_amount.0 - near_before.pool_amount.0,
        near(20)
    );
    assert_eq!(
        usdc_before.pool_amount.0 - usdc_after.pool_amount.0,
        dollars(100)
    );
}

#[test]
fn test_swap_near_with_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(500));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(20));

    let near_before = vcontract.get_asset_info(near_id());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 100,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract.swap_near(usdc_id(), None, None);

    let near_after = vcontract.get_asset_info(near_id());
    let usdc_after = vcontract.get_asset_info(usdc_id());
    let fee = vcontract.get_assets_fee();
    let usdc_fee = fee.iter().find(|fee| fee.asset_id == usdc_id()).unwrap();

    assert_eq!(
        near_after.pool_amount.0 - near_before.pool_amount.0,
        near(20)
    );
    // $100 * 1% = $1 fee. Removed from pool - $99 of output amount + $1 fee
    assert_eq!(
        usdc_before.pool_amount.0 - usdc_after.pool_amount.0,
        dollars(100)
    );
    assert_eq!(usdc_fee.usd.0, 1000000);
}

#[test]
#[should_panic(expected = "Swap is currently disabled")]
fn test_disabled_swap() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_swap_enabled(false);

    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}

#[test]
#[should_panic(expected = "Swap tokens should be different")]
fn test_same_token() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .swap(&AssetId::NEAR, &AssetId::NEAR, dollars(50), None);
}

#[test]
fn test_in_out_only() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::OutOnly,
        },
    );

    vcontract.set_asset_state(
        usdc_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::InOnly,
        },
    );

    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}

#[test]
fn test_disabled_swap_owner_call() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::Disabled,
        },
    );

    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}

#[test]
#[should_panic(expected = "Not enough liquidity to perform swap")]
fn test_not_enough_available_amount() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset = vcontract.get_asset_info("near".to_string());
    let available_amount = asset.available_liquidity.0 * dollars(5) / NEAR_DENOMINATION;
    vcontract.contract_mut().swap(
        &usdc_id().into(),
        &AssetId::NEAR,
        available_amount + 1,
        None,
    );
}

#[test]
#[should_panic]
fn test_disabled_swap_for_asset() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::Disabled,
        },
    );

    set_predecessor(&mut context, Bob);
    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}

#[test]
#[should_panic]
fn test_asset_in_disabled() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_asset_state(
        usdc_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::OutOnly,
        },
    );

    set_signer(&mut context, Alice);
    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}

#[test]
#[should_panic]
fn test_asset_out_disabled() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.add_price_oracle(get_account(Admin));

    // add liquidity to two assets
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::Enabled,
            lp_support: LpSupportState::Enabled,
            swap: SwapState::InOnly,
        },
    );

    set_signer(&mut context, Alice);
    vcontract
        .contract_mut()
        .swap(&usdc_id().into(), &AssetId::NEAR, dollars(50), None);
}
