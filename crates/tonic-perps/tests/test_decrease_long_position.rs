mod common;

use std::time::Duration;

use common::*;
use proptest::prelude::*;

#[test]
fn test_close_position_with_profit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    let asset_before_increase = vcontract.get_asset_info("near".to_string());

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before_decrease = vcontract.get_asset_info("near".to_string());
    assert_eq!(
        asset_before_decrease.pool_amount.0 - asset_before_increase.pool_amount.0,
        near(5)
    );

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let asset_after = vcontract.get_asset_info("near".to_string());

    assert_eq!(user_positions.len(), 0);
    // User receives his collateral back with the profit he get on price changes.
    // $25 collateral + $100 size * ($6 - $5) / $5 = $45.
    // In tokens $45 is 7.5NEAR
    assert_eq!(
        asset_before_decrease.pool_amount.0 - asset_after.pool_amount.0,
        75 * 10u128.pow(23)
    );
    assert_eq!(asset_after.available_liquidity, asset_after.pool_amount);
}

#[test]
fn test_decrease_position_with_profit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before = vcontract.get_asset_info("near".to_string());

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(10)),
        size_delta: U128(dollars(20)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let asset_after = vcontract.get_asset_info("near".to_string());

    assert_eq!(user_positions.len(), 1);
    // User receives his collateral back with the profit he get on price changes.
    // $10 collateral_delta + $20 size_delta * ($6 - $5) / $5 = $14.
    // In tokens $14 is 2.33NEAR
    assert_eq!(
        asset_before.pool_amount.0 - asset_after.pool_amount.0,
        2333333333333333333333333
    );
}

#[test]
fn test_close_position_with_loss() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_predecessor(&mut context, Admin);
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before = vcontract.get_asset_info("near".to_string());

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);
    // Will fail due to error with collateral calculations
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let asset_after = vcontract.get_asset_info("near".to_string());

    assert_eq!(user_positions.len(), 0);
    // User receives his collateral back minus his loss.
    // $50 collateral - $100 size * ($5 - $4) / $5 = $30.
    // In tokens $30 is 7.5NEAR
    assert_eq!(
        asset_before.pool_amount.0 - asset_after.pool_amount.0,
        7500000000000000000000000
    );
    assert_eq!(asset_after.pool_amount, asset_after.available_liquidity);
}

#[test]
fn test_decrease_position_with_loss() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(20));

    // Open a 4x leveraged position long NEAR
    set_predecessor(&mut context, Admin);
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before = vcontract.get_asset_info("near".to_string());

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(10)),
        size_delta: U128(dollars(60)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let asset_after = vcontract.get_asset_info("near".to_string());

    assert_eq!(user_positions.len(), 1);
    // $100 initial collateral - $88 remaining collateral = $12 loss + user gets $10 or 2.5NEAR
    assert_eq!(dollars(100) - user_positions[0].collateral.0, dollars(22));
    assert_eq!(
        asset_before.pool_amount.0 - asset_after.pool_amount.0,
        2500000000000000000000000
    );
}

#[test]
#[should_panic(expected = "Losses exceed collateral")]
fn test_close_long_position_collateral_loss() {
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

    update_near_price(&mut vcontract, dollars(3));
    let status = vcontract.get_liquidation_status(&position_id);
    assert!(status.insolvent);

    set_deposit(&mut context, 1);

    // User's loss on this position is $40, however he has a collateral
    // in amount of $25. It means user that position should be liquidated
    // instead of decreased.
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
fn test_close_long_position_with_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    set_predecessor(&mut context, Admin);
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before = vcontract.get_asset_info("near".to_string());

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 100,
    });

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(0)),
        size_delta: U128(dollars(60)),
        referrer_id: None,
        output_token_id: None,
    });

    let asset_after = vcontract.get_asset_info("near".to_string());
    let fee = vcontract.get_assets_fee();
    let near_fee = fee.iter().find(|fee| fee.asset_id == near_id()).unwrap();

    // Collateral reduction = $12, fees = $0.6. The removed amount of fee 0.015NEAR
    assert_eq!(
        asset_before.pool_amount.0 - asset_after.pool_amount.0,
        150000000000000000000000
    );
    assert_eq!(near_fee.token_amount, 150000000000000000000000.into());
}

#[test]
fn test_reduce_only_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    let asset_before_increase = vcontract.get_asset_info("near".to_string());

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before_decrease = vcontract.get_asset_info("near".to_string());
    assert_eq!(
        asset_before_decrease.pool_amount.0 - asset_before_increase.pool_amount.0,
        near(5)
    );

    update_near_price(&mut vcontract, dollars(6));
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::ReduceOnly,
            lp_support: LpSupportState::Disabled,
            swap: SwapState::Disabled,
        },
    );

    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
#[should_panic(expected = "Can not decrease other account's position")]
fn test_decrease_position_wrong_account() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(6));
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(0),
        size_delta: U128(dollars(50)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
#[should_panic]
fn test_decrease_position_disabled_asset() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(6));
    vcontract.disable_asset(near_id());
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(0),
        size_delta: U128(dollars(50)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
fn test_decrease_position_zero_delta() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(20));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(150)),
        is_long: true,
        referrer_id: None,
    });

    let user_positions_before = vcontract.get_positions(get_account(Admin));

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(0)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions_after = vcontract.get_positions(get_account(Admin));

    assert_eq!(user_positions_after.len(), 1);
    // User decrease his collateral on this position in amount of $25
    assert_eq!(
        user_positions_before[0].collateral.0 - user_positions_after[0].collateral.0,
        dollars(25)
    );
}

#[test]
#[should_panic(expected = "Margin level is less than allowed")]
fn test_decrease_position_min_margin_level() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(100));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(1000)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(27) / 10);
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(0),
        size_delta: U128(dollars(20)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
#[should_panic(expected = "Contract is temporary paused")]
fn test_decrease_position_paused_contract_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);

    vcontract.set_state(ContractState::Paused);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(20),
        size_delta: U128(dollars(90)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
#[should_panic(expected = "Position leverage is lower than minimum leverage")]
fn test_decrease_position_below_min_leverage() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(10));

    set_deposit(&mut context, near(100));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(1000)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(9));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(0),
        size_delta: U128(dollars(10)),
        referrer_id: None,
        output_token_id: None,
    });
}

#[test]
fn test_min_profit_time() {
    let (mut context, vcontract) = setup();

    let last_increased_time =
        Duration::from_nanos(context.context.block_timestamp).as_millis() as u64;

    // Default min profit time is 60sec, add 59sec
    context.block_timestamp(
        context.context.block_timestamp + Duration::from_secs(59).as_nanos() as u64,
    );
    testing_env!(context.build());

    let mut asset = Asset::new(AssetId::NEAR, 24, false, 50, 0);
    asset.price = dollars(55) / 10;
    asset.min_profit_bps = 1000;

    let (has_profit, delta) =
        vcontract
            .contract()
            .get_delta(&asset, dollars(100), dollars(5), true, last_increased_time);

    assert_eq!(delta, 0);
    assert!(has_profit);

    let last_increased_time =
        Duration::from_nanos(context.context.block_timestamp).as_millis() as u64;

    // Default min profit time is 60sec, add 61sec
    context.block_timestamp(
        context.context.block_timestamp + Duration::from_secs(61).as_nanos() as u64,
    );
    testing_env!(context.build());

    let (has_profit, delta) =
        vcontract
            .contract()
            .get_delta(&asset, dollars(100), dollars(5), true, last_increased_time);

    assert_eq!(delta, 10000000);
    assert!(has_profit);
}

#[test]
fn test_close_position_with_output_token() {
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
    let asset_before_increase = vcontract.get_asset_info("near".to_string());

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before_decrease = vcontract.get_asset_info("near".to_string());
    assert_eq!(
        asset_before_decrease.pool_amount.0 - asset_before_increase.pool_amount.0,
        near(5)
    );

    update_near_price(&mut vcontract, dollars(6));

    // The position is long so collateral is in NEAR, set USDC token as output token
    let transfer_info = vcontract.contract_mut().decrease_position(
        position_id,
        dollars(25),
        dollars(100),
        None,
        false,
        Some(usdc_id()),
    );

    // Collateral $25 + profit $20
    assert_eq!(transfer_info.amount(), dollars(45));
    assert_eq!(transfer_info.asset_id(), usdc_id().into());
}

#[test]
fn test_same_collateral_and_output_token() {
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
    let asset_before_increase = vcontract.get_asset_info("near".to_string());

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset_before_decrease = vcontract.get_asset_info("near".to_string());
    assert_eq!(
        asset_before_decrease.pool_amount.0 - asset_before_increase.pool_amount.0,
        near(5)
    );

    update_near_price(&mut vcontract, dollars(6));

    // The position is long so collateral is in NEAR, set USDC token as output token
    let transfer_info = vcontract.contract_mut().decrease_position(
        position_id,
        dollars(25),
        dollars(100),
        None,
        false,
        Some(near_id()),
    );

    // Collateral $25 + profit $20 = 7.5NEAR
    assert_eq!(transfer_info.amount(), near(75) / 10);
    assert_eq!(transfer_info.asset_id(), near_id().into());
}

#[test]
#[should_panic(
    expected = "Not enough collateral to cover losses and send tokens out. 
            Close the position or specify less collateral delta"
)]
fn test_close_position_exceeded_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Admin);
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(46) / 10);
    set_deposit(&mut context, 1);

    // Losses = $80 * ($5 - $4.6) / $5 = $6.4. Collateral delta = $44.
    // Collateral reduction $50.4 > collateral $50
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(44)),
        size_delta: U128(dollars(80)),
        referrer_id: None,
        output_token_id: None,
    });
}

proptest! {
    #[test]
    fn test_long_position(
        decrease_position_size in dollars(0)..dollars(50),
        collateral_amount in dollars(30)..dollars(40),
        second_price in dollars(4)..dollars(6)
    ) {
        let (mut context, mut vcontract) = setup();
        set_predecessor(&mut context, Admin);

        // add liquidity to NEAR
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::NEAR, near(1000));

        let first_price = dollars(5);

        update_near_price(&mut vcontract, first_price);

        set_deposit(&mut context, near(20));
        let position_id = vcontract.increase_position(IncreasePositionRequest {
            underlying_id: "near".to_string(),
            size_delta: U128(dollars(150)),
            is_long: true,
            referrer_id: None,

        });

        let asset_before = vcontract.get_asset_info("near".to_string());
        let user_positions_before = vcontract.get_positions(get_account(Admin));

        update_near_price(&mut vcontract, second_price);
        set_deposit(&mut context, 1);

        vcontract.decrease_position(DecreasePositionRequest {
            position_id: position_id,
            collateral_delta: U128(collateral_amount),
            size_delta: U128(decrease_position_size),
            referrer_id: None, output_token_id: None,

        });

        let user_positions_after = vcontract.get_positions(get_account(Admin));
        let asset_after = vcontract.get_asset_info("near".to_string());
        let collateral_change = if first_price >= second_price {
            collateral_amount
        } else {
            let price_change = second_price - first_price;
            let profit = decrease_position_size * price_change / first_price;
            collateral_amount + profit
        };

        let collateral_reduction = if first_price >= second_price {
            let price_change = first_price - second_price;
            let loss = decrease_position_size * price_change / first_price;
            collateral_amount + loss
        } else {
            collateral_amount
        };

        let collateral_in_tokens =
            collateral_change * 10u128.pow(asset_after.decimals as u32) / second_price;

        assert_eq!(
            user_positions_before[0].collateral.0 - user_positions_after[0].collateral.0,
            collateral_reduction
        );
        assert_eq!(
            asset_before.pool_amount.0 - asset_after.pool_amount.0,
            collateral_in_tokens
        );

        vcontract.decrease_position(DecreasePositionRequest {
            position_id: position_id,
            collateral_delta: U128(0),
            size_delta: U128(user_positions_after[0].size.0),
            referrer_id: None, output_token_id: None,

        });

        vcontract.remove_admin(get_account(Admin));
    }

    #[test]
    fn test_multiple_decrease_position(
        swap_iteration in 5..10,
        decrease_position_size in dollars(10)..dollars(20),
        collateral_amount in dollars(5)..dollars(15),
        first_price in dollars(4)..dollars(10),
        second_price in dollars(4)..dollars(10),
    ) {
        let (mut context, mut vcontract) = setup();
        set_predecessor(&mut context, Admin);

        // add liquidity to NEAR
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::NEAR, near(1000));

        update_near_price(&mut vcontract, first_price);

        let near_deposit = near(150);
        set_deposit(&mut context, near_deposit);
        let position_id = vcontract.increase_position(IncreasePositionRequest {
            underlying_id: "near".to_string(),
            size_delta: U128(dollars(1650)),
            is_long: true,
            referrer_id: None,

        });

        let asset_before = vcontract.get_asset_info("near".to_string());

        update_near_price(&mut vcontract, second_price);

        for _ in 4..=swap_iteration{
            // Use internal decrease position so to avoid
            // BalanceExceeded error while transferring the tokens
            vcontract.contract_mut().decrease_position(
                position_id,
                collateral_amount,
                decrease_position_size,
                None,
                false,
                None
            );
        }

        let user_positions = vcontract.get_positions(get_account(Admin));
        assert_eq!(user_positions.len(), 1);

        vcontract.contract_mut().decrease_position(
            position_id,
            0,
            user_positions[0].size.0,
            None,
            false,
            None
        );

        let asset_after = vcontract.get_asset_info("near".to_string());
        let collateral_amount = near_deposit * first_price / second_price;
        // Check that in case of users loss pool gets part of his collateral,
        // in case of users profit pool loses more than only just his collateral
        if first_price >= second_price {
            assert!(asset_after.pool_amount.0 >= asset_before.pool_amount.0 - collateral_amount);
        } else {
            assert!(asset_after.pool_amount.0 < asset_before.pool_amount.0 - collateral_amount);
        };

        vcontract.remove_admin(get_account(Admin));
    }
}
