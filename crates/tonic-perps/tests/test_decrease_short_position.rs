mod common;

use common::*;
use proptest::prelude::*;

#[test]
fn test_close_position_with_profit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

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

    let near_before = vcontract.get_asset_info("near".to_string());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(25)),
        size_delta: U128(dollars(100)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info("near".to_string());
    let usdc_after = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions.len(), 0);
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.available_liquidity, near_after.pool_amount);
    // Protocol loses $20 on price changes: $100 size * ($5 - $4) / $5 = $20
    assert_eq!(
        usdc_before.pool_amount.0 - usdc_after.pool_amount.0,
        dollars(20)
    );
}

#[test]
fn test_decrease_position_with_profit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position short NEAR
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    let near_before = vcontract.get_asset_info("near".to_string());
    let usdc_before = vcontract.get_asset_info(usdc_id());

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(10)),
        size_delta: U128(dollars(20)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info("near".to_string());
    let usdc_after = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions.len(), 1);
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.available_liquidity, near_after.pool_amount);
    // Protocol loses $20 on price changes: $20 size_delta * ($5 - $4) / $5 = $4
    assert_eq!(
        usdc_before.pool_amount.0 - usdc_after.pool_amount.0,
        dollars(4)
    );
}

#[test]
fn test_check_short_open_interest() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position short NEAR
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(120)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(10)),
        size_delta: U128(dollars(20)),
        referrer_id: None,
        output_token_id: None,
    });

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(100),
            short: dollars(300),
        },
    );

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });
}

#[test]
fn test_close_position_with_loss() {
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
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    let near_before = vcontract.get_asset_info("near".to_string());
    let usdc_before = vcontract.get_asset_info(usdc_id());

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
    let near_after = vcontract.get_asset_info("near".to_string());
    let usdc_after = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions.len(), 0);
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.pool_amount.0, near_after.available_liquidity.0);
    // Protocol makes profit $20 on price changes: $100 size * ($6 - $5) / $5 = $20
    assert_eq!(
        usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
        dollars(20)
    );
}

#[test]
fn test_decrease_position_with_loss() {
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
    set_deposit(&mut context, near(20));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: false,
        referrer_id: None,
    });

    let near_before = vcontract.get_asset_info("near".to_string());
    let usdc_before = vcontract.get_asset_info(usdc_id());

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
    let near_after = vcontract.get_asset_info("near".to_string());
    let usdc_after = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions.len(), 1);
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.pool_amount.0, near_after.available_liquidity.0);
    // Protocol makes profit $4 on price changes: $20 size_delta * ($6 - $5) / $5 = $4
    assert_eq!(
        usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
        dollars(4)
    );
}

#[test]
fn test_decrease_position_no_usd_out() {
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
    set_deposit(&mut context, near(20));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: false,
        referrer_id: None,
    });
    let near_before = vcontract.get_asset_info("near".to_string());
    let usdc_before = vcontract.get_asset_info(usdc_id());
    let user_positions_before = vcontract.get_positions(get_account(Admin));

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(0)),
        size_delta: U128(dollars(20)),
        referrer_id: None,
        output_token_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    let near_after = vcontract.get_asset_info("near".to_string());
    let usdc_after = vcontract.get_asset_info(usdc_id());

    assert_eq!(user_positions.len(), 1);
    // Users loses $4 on price changes: $20 size_delta * ($6 - $5) / $5 = $4
    assert_eq!(
        user_positions_before[0].collateral.0 - user_positions[0].collateral.0,
        dollars(4)
    );
    assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
    assert_eq!(near_after.pool_amount.0, near_after.available_liquidity.0);
    // Protocol makes profit $4 on price changes: $20 size_delta * ($6 - $5) / $5 = $4
    assert_eq!(
        usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
        dollars(4)
    );
}

#[test]
#[should_panic(expected = "Losses exceed collateral")]
fn test_close_position_no_usd_out() {
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
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(12));
    set_deposit(&mut context, 1);

    let status = vcontract.get_liquidation_status(&position_id);
    assert!(status.insolvent);

    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(0)),
        size_delta: U128(dollars(90)),
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
fn test_decrease_position_zero_delta() {
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
    set_deposit(&mut context, near(20));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(150)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(6));
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        position_id: position_id,
        collateral_delta: U128(dollars(10)),
        size_delta: U128(dollars(0)),
        referrer_id: None,
        output_token_id: None,
    });
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

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(10000));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position short NEAR
    set_deposit(&mut context, near(100));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(1000)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(73) / 10);
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
fn test_close_position_with_output_token() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

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

    update_near_price(&mut vcontract, dollars(4));

    // The position is short so collateral is in USDC, set NEAR token as output token
    let transfer_info = vcontract.contract_mut().decrease_position(
        position_id,
        dollars(25),
        dollars(100),
        None,
        false,
        Some(near_id()),
    );

    // Collateral $25 + profit $20 = $45 at price $4 = 11.25 NEAR
    assert_eq!(transfer_info.amount(), near(1125) / 100);
    assert_eq!(transfer_info.asset_id(), near_id().into());
}

proptest! {
    #[test]
    fn test_short_position(
        decrease_position_size in dollars(0)..dollars(100),
        collateral_amount in dollars(0)..dollars(30),
        second_price in dollars(4)..dollars(6),
        margin_fee_bps in 0u16..10,
    ) {
        let (mut context, mut vcontract) = setup();
        set_predecessor(&mut context, Admin);

        // add liquidity to NEAR
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::NEAR, near(1000));

        // add liquidity to USDC
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

        let first_price = dollars(5);

        update_near_price(&mut vcontract, first_price);

        // Open a 4x leveraged position short NEAR
        set_deposit(&mut context, near(20));
        let position_id = vcontract.increase_position(IncreasePositionRequest {
            underlying_id: "near".to_string(),
            size_delta: U128(dollars(200)),
            is_long: false,
            referrer_id: None,

        });

        let near_before = vcontract.get_asset_info("near".to_string());
        let usdc_before = vcontract.get_asset_info(usdc_id());

        update_near_price(&mut vcontract, second_price);
        vcontract.set_fee_parameters(FeeParameters {
            tax_bps: 0,
            stable_tax_bps: 0,
            mint_burn_fee_bps: 0,
            swap_fee_bps: 0,
            stable_swap_fee_bps: 0,
            margin_fee_bps,
        });

        set_deposit(&mut context, 1);

        vcontract.decrease_position(DecreasePositionRequest {
            position_id: position_id,
            collateral_delta: U128(collateral_amount),
            size_delta: U128(decrease_position_size),
            referrer_id: None, output_token_id: None,

        });

        let user_positions = vcontract.get_positions(get_account(Admin));
        let near_after = vcontract.get_asset_info("near".to_string());
        let usdc_after = vcontract.get_asset_info(usdc_id());

        let price_change = if first_price >= second_price {
                first_price - second_price
        } else {
                second_price - first_price
        };
        let pool_change = decrease_position_size * price_change / first_price;

        assert_eq!(user_positions.len(), 1);
        assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
        assert_eq!(near_after.available_liquidity, near_after.pool_amount);
        if first_price >= second_price {
            assert_eq!(
                usdc_before.pool_amount.0 - usdc_after.pool_amount.0,
                pool_change
            );
        } else {
            assert_eq!(
                usdc_after.pool_amount.0 - usdc_before.pool_amount.0,
                pool_change
            );
        }

        vcontract.decrease_position(DecreasePositionRequest {
            position_id: position_id,
            collateral_delta: U128(0),
            size_delta: U128(user_positions[0].size.0),
            referrer_id: None, output_token_id: None,

        });

        vcontract.remove_admin(get_account(Admin));
    }

    #[test]
    fn test_multiple_decrease_position(
        swap_iteration in 5..10,
        decrease_position_size in dollars(10)..dollars(20),
        collateral_amount in dollars(5)..dollars(15),
        second_price in dollars(4)..dollars(10),
        margin_fee_bps in 0u16..10,
    ) {
        let (mut context, mut vcontract) = setup();
        set_predecessor(&mut context, Admin);

        // add liquidity to NEAR
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::NEAR, near(1000));

        // add liquidity to USDC
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(10000));

        let first_price = dollars(7);
        update_near_price(&mut vcontract, first_price);

        set_deposit(&mut context, near(200));
        let position_id = vcontract.increase_position(IncreasePositionRequest {
            underlying_id: "near".to_string(),
            size_delta: U128(dollars(1600)),
            is_long: false,
            referrer_id: None,

        });

        let near_before = vcontract.get_asset_info("near".to_string());
        let usdc_before = vcontract.get_asset_info(usdc_id());

        update_near_price(&mut vcontract, second_price);
        vcontract.set_fee_parameters(FeeParameters {
            tax_bps: 0,
            stable_tax_bps: 0,
            mint_burn_fee_bps: 0,
            swap_fee_bps: 0,
            stable_swap_fee_bps: 0,
            margin_fee_bps,
        });

        for _ in 4..=swap_iteration {
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
        let near_after = vcontract.get_asset_info("near".to_string());
        let usdc_after = vcontract.get_asset_info(usdc_id());

        assert_eq!(user_positions.len(), 1);
        assert_eq!(near_before.pool_amount.0, near_after.pool_amount.0);
        assert_eq!(near_after.available_liquidity, near_after.pool_amount);

        vcontract.contract_mut().decrease_position(
            position_id,
            0,
            user_positions[0].size.0,
            None,
            false,
            None
        );

        // Check that in case of users loss pool gets part of his collateral,
        // in case of users profit pool loses users profit
        if first_price >= second_price {
            assert!(usdc_before.pool_amount.0 >= usdc_after.pool_amount.0);
        } else {
            assert!(usdc_before.pool_amount.0 < usdc_after.pool_amount.0);
        }

        vcontract.remove_admin(get_account(Admin));
    }
}
