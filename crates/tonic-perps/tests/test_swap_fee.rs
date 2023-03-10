mod common;

use common::*;

#[test]
fn test_decrease_near_fee() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(5));

    // Open a 4x leveraged position long NEAR
    set_predecessor(&mut context, Admin);
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 10,
    });

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let near_before = vcontract.get_asset_info(near_id());
    let fee = vcontract.get_assets_fee();
    let near_fee_before = fee.iter().find(|fee| fee.asset_id == near_id()).unwrap();
    // Convertion loss. NEAR price = $3. Position size $100 * 0.1% = $0.1 = 0.025NEAR
    assert_eq!(near_fee_before.token_amount, 25000000000000000000000.into());

    let near_after = vcontract.get_asset_info(near_id());
    let transfer_amount = vcontract.withdraw_fees(Some(vec![near_id()]));
    let fee = vcontract.get_assets_fee();
    let near_fee_after = fee.iter().find(|fee| fee.asset_id == near_id()).unwrap();
    // Transfer 0.33NEAR = 0.1$
    assert_eq!(transfer_amount, 100000.into());
    // Remaining NEAR fee = $0
    assert_eq!(near_fee_after.token_amount, 0.into());
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
}

#[test]
fn test_decrease_usdc_fee() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position short NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 10,
    });

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let usdc_before = vcontract.get_asset_info(usdc_id());
    let fee = vcontract.get_assets_fee();
    let usdc_fee = fee.iter().find(|fee| fee.asset_id == usdc_id()).unwrap();
    // Position size $100 * 0.1% = $0.1
    assert_eq!(usdc_fee.token_amount, 100000.into());

    let fee_transfer = vcontract.withdraw_fees(Some(vec![usdc_id()]));
    let fee = vcontract.get_assets_fee();
    let usdc_after = vcontract.get_asset_info(usdc_id());
    // Transfer $0.1
    assert_eq!(fee_transfer, 100000.into());
    assert_eq!(fee[0].token_amount, 0.into());
    assert_eq!(usdc_after.pool_amount, usdc_before.pool_amount);
}

#[test]
fn test_decrease_all_fee() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position short NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 10,
    });

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let near_before = vcontract.get_asset_info(near_id());
    let usdc_before = vcontract.get_asset_info(usdc_id());
    let fee = vcontract.get_assets_fee();
    let usdc_fee = fee.iter().find(|fee| fee.asset_id == usdc_id()).unwrap();
    let near_fee = fee.iter().find(|fee| fee.asset_id == near_id()).unwrap();
    let total_fee_usd = usdc_fee.usd.0 + near_fee.usd.0;
    // Open NEAR price = $6. Position size $100 * 0.1% = $0.1 = 0.016NEAR
    // Close NEAR price = $5. Position size $100 * 0.1% = $0.1 = 0.02NEAR
    // Total fee = 0.016NEAR + 0.02NEAR = 0.036NEAR
    assert_eq!(near_fee.token_amount.0, 36666666666666666666666);
    // Position size $100 * 0.1% = $0.1
    assert_eq!(usdc_fee.token_amount.0, 100000);
    assert_eq!(total_fee_usd, 283333);

    let fee_transfer = vcontract.withdraw_fees(None);

    let fee = vcontract.get_assets_fee();
    let usdc_fee = fee.iter().find(|fee| fee.asset_id == usdc_id()).unwrap();
    let near_fee = fee.iter().find(|fee| fee.asset_id == near_id()).unwrap();
    let near_after = vcontract.get_asset_info(near_id());
    let usdc_after = vcontract.get_asset_info(usdc_id());
    // NEAR price = $5. Transfer 0.03666NEAR * $5 + $0.1 = $0.18 + $0.1 = $0.28
    assert_eq!(fee_transfer.0, 283333);
    assert_eq!(usdc_fee.token_amount.0, 0);
    assert_eq!(near_fee.token_amount.0, 0);
    assert_eq!(usdc_after.pool_amount, usdc_before.pool_amount);
    assert_eq!(near_before.pool_amount, near_after.pool_amount);
}
