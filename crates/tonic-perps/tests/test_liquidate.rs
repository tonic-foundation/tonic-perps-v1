mod common;

use common::*;

#[test]
#[should_panic(expected = "Position not eligible for liquidation")]
fn test_liquidate_solvent_position() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));
    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);

    vcontract.liquidate_position(LiquidatePositionRequest { position_id });
}

#[test]
#[should_panic(expected = "Contract is temporary paused")]
fn test_liquidate_paused_contract() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));
    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    vcontract.set_state(ContractState::Paused);
    set_deposit(&mut context, 1);

    vcontract.liquidate_position(LiquidatePositionRequest { position_id });
}

#[test]
fn test_liquidate_leverage_position() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200000));
    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(60));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(2000)),
        is_long: true,
        referrer_id: None,
    });

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 10,
        stable_tax_bps: 10,
        mint_burn_fee_bps: 10,
        swap_fee_bps: 10,
        stable_swap_fee_bps: 10,
        margin_fee_bps: 10,
    });

    update_near_price(&mut vcontract, dollars(45) / 10);
    let position = vcontract.get_position(&position_id).unwrap();
    let old_size = position.size.0;
    assert_eq!(
        old_size,
        vcontract.get_position(&position_id).unwrap().size.0
    );

    vcontract.add_admin(get_account(Alice), AdminRole::Liquidator);
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);
    vcontract.liquidate_position(LiquidatePositionRequest { position_id });

    assert!(old_size > vcontract.get_position(&position_id).unwrap().size.0);
}

#[test]
fn test_get_liquidation_status() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200000));
    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(60));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(2000)),
        is_long: true,
        referrer_id: None,
    });

    let (min_leverage, max_leverage) = (vcontract.get_min_leverage(), vcontract.get_max_leverage());

    let status = vcontract.get_liquidation_status(&position_id);
    assert!(status.leverage >= min_leverage && status.leverage <= max_leverage);
    assert!(!status.max_leverage_exceeded);
    assert!(!status.insolvent);

    update_near_price(&mut vcontract, dollars(45) / 10);
    let status = vcontract.get_liquidation_status(&position_id);
    // Liquidation leverage = max leverage 11x + 25% = 13.75x.
    assert!(status.leverage > 13750);
    assert!(status.max_leverage_exceeded);
    assert!(!status.insolvent);

    update_near_price(&mut vcontract, dollars(47) / 10);
    let status = vcontract.get_liquidation_status(&position_id);
    // Position leverage exceeds max leverage but doesn't reaches liquidation leverage.
    assert!(status.leverage < 13750);
    assert!(status.leverage > max_leverage);
    assert!(!status.max_leverage_exceeded);
    assert!(!status.insolvent);

    update_near_price(&mut vcontract, dollars(3));
    let status = vcontract.get_liquidation_status(&position_id);
    assert_eq!(status.leverage, 0);
    assert!(!status.max_leverage_exceeded);
    assert!(status.insolvent);
}

#[test]
fn test_liquidate_position_short() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(200000));
    update_near_price(&mut vcontract, dollars(1));

    set_deposit(&mut context, near(25));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(250)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(20));

    let status = vcontract.get_liquidation_status(&position_id);
    let user_positions_before = vcontract.get_positions(get_account(Admin));
    let near_before = vcontract.get_asset_info(near_id());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions_before[0].value.0, 0);
    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    set_predecessor(&mut context, Alice);

    let (_, _, liquidator_transfer_info) = vcontract.contract_mut().liquidate_position(position_id);

    let user_positions_after = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info(near_id());
    let usdc_after = vcontract.get_asset_info(usdc_id());
    let reward = liquidator_transfer_info.unwrap().amount();

    assert_eq!(user_positions_after.len(), 0);
    // Reward - $25 of collateral * 10% of reward = $2.5
    assert_eq!(reward, dollars(25) / 10);
    assert!(vcontract.get_positions(get_account(Admin)).is_empty());

    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.available_liquidity, near_after.pool_amount);
    // Protocol gains remaining collateral $40 and losses $4 of reward
    assert_eq!(
        usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
        dollars(25) - reward
    );
}

#[test]
fn test_liquidate_position_long() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    update_near_price(&mut vcontract, dollars(20));

    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(2000)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(1));

    let status = vcontract.get_liquidation_status(&position_id);
    let near_before = vcontract.get_asset_info(near_id());

    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    set_predecessor(&mut context, Alice);

    let (_, _, liquidator_transfer_info) = vcontract.contract_mut().liquidate_position(position_id);

    let user_positions_after = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info(near_id());
    let reward = liquidator_transfer_info.unwrap().amount();

    assert_eq!(user_positions_after.len(), 0);
    // Reward = $200 of collateral * 10% of reward = $20 / 20NEAR
    assert_eq!(reward, near(20));
    // Protocol losses only reward as collateral was stored there.
    assert_eq!(near_before.pool_amount.0 - near_after.pool_amount.0, reward);
}

#[test]
fn test_liquidate_position_max_reward() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    update_near_price(&mut vcontract, dollars(20));

    set_deposit(&mut context, near(50));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(2000)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(1));

    let status = vcontract.get_liquidation_status(&position_id);

    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    set_predecessor(&mut context, Alice);

    let (_, _, liquidator_transfer_info) = vcontract.contract_mut().liquidate_position(position_id);

    let user_positions_after = vcontract.get_positions(get_account(Admin));
    let reward = liquidator_transfer_info.unwrap().amount();

    assert_eq!(user_positions_after.len(), 0);
    // Reward = $1000 of collateral * 10% of reward = $100. Max reward amount is $25 / 25 NEAR
    assert_eq!(reward, near(25));
}

#[test]
#[should_panic(expected = "caller must be have a Liquidator role")]
fn test_liquidate_not_admin_in_private_liquidation_mode() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(200000));
    update_near_price(&mut vcontract, dollars(1));

    set_deposit(&mut context, near(25));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(250)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(20));

    let status = vcontract.get_liquidation_status(&position_id);

    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    vcontract.set_private_liquidation_only(true);

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);
    vcontract.liquidate_position(LiquidatePositionRequest { position_id });

    let user_positions = vcontract.get_positions(get_account(Admin));

    assert_eq!(user_positions.len(), 0);
    assert!(vcontract.get_positions(get_account(Admin)).is_empty());
}

#[test]
fn test_liquidate_in_private_liquidation_mode() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(200000));
    update_near_price(&mut vcontract, dollars(1));

    set_deposit(&mut context, near(25));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(250)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(20));

    let status = vcontract.get_liquidation_status(&position_id);
    let user_positions = vcontract.get_positions(get_account(Admin));

    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);
    assert_eq!(user_positions[0].value.0, 0);

    vcontract.set_private_liquidation_only(true);
    vcontract.add_admin(get_account(Alice), AdminRole::Liquidator);
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);

    vcontract.liquidate_position(LiquidatePositionRequest { position_id });

    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_positions.len(), 0);
}

#[test]
fn test_liquidate_position_fees_exceed_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(200));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(200000));
    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: U128(dollars(250)),
        is_long: false,
        referrer_id: None,
    });

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 500,
    });

    update_near_price(&mut vcontract, dollars(58) / 10);

    let status = vcontract.get_liquidation_status(&position_id);
    let user_positions_before = vcontract.get_positions(get_account(Admin));
    let near_before = vcontract.get_asset_info(near_id());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    // $250 size - $40 loss = $210
    assert_eq!(user_positions_before[0].value.0, dollars(210));
    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    set_predecessor(&mut context, Alice);

    let (_, _, liquidator_transfer_info) = vcontract.contract_mut().liquidate_position(position_id);

    let user_positions_after = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info(near_id());
    let usdc_after = vcontract.get_asset_info(usdc_id());
    let reward = liquidator_transfer_info.unwrap().amount();

    assert_eq!(user_positions_after.len(), 0);
    // Reward - ($50 of collateral - $10 fee) * 10% of reward = $2.5
    assert_eq!(reward, dollars(4));
    assert!(vcontract.get_positions(get_account(Admin)).is_empty());

    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.available_liquidity, near_after.pool_amount);
    // Protocol gains remaining collateral $40 and losses $4 of reward
    assert_eq!(
        usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
        dollars(40) - reward
    );
}
