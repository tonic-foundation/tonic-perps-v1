mod common;

use common::*;

#[test]
fn test_get_fee_parameters() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    let fee_params = FeeParameters {
        tax_bps: 1,
        stable_tax_bps: 2,
        mint_burn_fee_bps: 3,
        swap_fee_bps: 4,
        stable_swap_fee_bps: 5,
        margin_fee_bps: 6,
    };

    vcontract.set_fee_parameters(fee_params.clone());

    let fee_params_view = vcontract.get_fee_parameters();

    assert_eq!(
        serde_json::ser::to_string(&fee_params).unwrap(),
        serde_json::ser::to_string(&fee_params_view).unwrap()
    );
}

#[test]
fn test_is_admin() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    assert!(vcontract.is_admin(get_account(Admin)));
    assert!(!vcontract.is_admin(get_account(Alice)));
}

#[test]
fn test_is_liquidator() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    assert!(vcontract.is_admin(get_account(Admin)));
    assert!(!vcontract.is_admin(get_account(Alice)));
}

#[test]
fn test_get_positions_for_asset() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(150));

    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    let positions = vcontract.get_positions_for_asset(get_account(Admin), "near".to_string());

    assert_eq!(positions.len(), 2);

    let short_position = positions.iter().find(|p| !p.is_long);
    let long_position = positions.iter().find(|p| p.is_long);

    assert!(short_position.is_some());
    assert!(long_position.is_some());

    let short_position = short_position.unwrap();
    let long_position = long_position.unwrap();

    assert_eq!(short_position.size.0, dollars(100));
    assert_eq!(long_position.size.0, dollars(100));

    assert_eq!(short_position.collateral.0, dollars(50));
    assert_eq!(long_position.collateral.0, dollars(50));

    assert_eq!(short_position.collateral_id, usdc_id());
    assert_eq!(long_position.collateral_id, near_id());
}

#[test]
fn test_get_position_by_id() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    set_deposit(&mut context, near(10));
    let pos_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let position = vcontract.get_position_by_id(pos_id.into());

    assert!(position.is_some());

    let position = position.unwrap();

    assert_eq!(position.size.0, dollars(100));
    assert_eq!(position.collateral.0, dollars(50));
    assert_eq!(position.collateral_id, near_id());
}

#[test]
fn test_get_liquidation_status() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(550));

    set_deposit(&mut context, near(10));
    let pos_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(500)),
        is_long: false,
        referrer_id: None,
    });

    let liquidation_status = vcontract.get_liquidation_status(&pos_id);

    assert!(!liquidation_status.insolvent);
    assert!(!liquidation_status.max_leverage_exceeded);
    assert!(liquidation_status.reason.is_none());

    update_near_price(&mut vcontract, dollars(50));

    let liquidation_status = vcontract.get_liquidation_status(&pos_id);

    assert!(liquidation_status.insolvent);
    assert!(!liquidation_status.max_leverage_exceeded);
    assert!(liquidation_status.reason.is_some());

    update_near_price(&mut vcontract, dollars(5));

    let fee_params = FeeParameters {
        tax_bps: 1,
        stable_tax_bps: 2,
        mint_burn_fee_bps: 3,
        swap_fee_bps: 4,
        stable_swap_fee_bps: 5,
        margin_fee_bps: 500,
    };

    vcontract.set_fee_parameters(fee_params);

    context.block_timestamp(std::time::Duration::from_secs(60 * 60 * 24).as_nanos() as u64);

    testing_env!(context.build());

    let liquidation_status = vcontract.get_liquidation_status(&pos_id);

    assert!(!liquidation_status.insolvent);
    assert!(liquidation_status.max_leverage_exceeded);
}

#[test]
fn test_get_mint_burn_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(500));

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 10,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    let fees = vcontract.get_mint_burn_fees(usdc_id(), U128(dollars(100)));

    assert_eq!(fees.mint_fee_bps, 10);
    assert_eq!(fees.burn_fee_bps, 10);
}

#[test]
fn test_get_mint_burn_fees_2() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(300));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(20));

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 10,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 10,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract.set_dynamic_swap_fees(true);

    let fees = vcontract.get_mint_burn_fees(usdc_id(), U128(dollars(100)));

    assert_eq!(fees.mint_fee_bps, 17);
    assert_eq!(fees.burn_fee_bps, 5);

    let fees = vcontract.get_mint_burn_fees(near_id(), U128(near(10)));

    assert_eq!(fees.mint_fee_bps, 5);
    assert_eq!(fees.burn_fee_bps, 16);
}
