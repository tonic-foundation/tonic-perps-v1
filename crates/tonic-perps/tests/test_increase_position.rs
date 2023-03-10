mod common;

use common::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::AccountId;

#[test]
fn test_open_position() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Add collateral for the new position
    // increase_near_position <- for near only
    // on_ft_transfer <-
    // msg = increase_position
    //   - add_collateral
    //   -

    // Open a 5x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    // println!("deposit: {}", near_deposit);

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
fn test_open_long_position_average_price() {
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

    let position_initial = vcontract.get_position(&position_id).unwrap();
    assert_eq!(dollars(5), position_initial.average_price.0);

    update_near_price(&mut vcontract, dollars(4));

    // Set size delta > 0
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let position_after_loss = vcontract.get_position(&position_id).unwrap();
    // Position price changes downwards $4.44, asset price = $4
    assert_eq!(position_after_loss.average_price.0, 4444444);

    // Update price to $18
    update_near_price(&mut vcontract, dollars(18));

    // Set size delta = 0
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(0)),
        is_long: true,
        referrer_id: None,
    });

    let position = vcontract.get_position(&position_id).unwrap();
    // Price remains the same = $4.44, asset price = $18
    assert_eq!(
        position_after_loss.average_price.0,
        position.average_price.0
    );

    // Update price to $6
    update_near_price(&mut vcontract, dollars(6));

    // Set size delta > 0
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let position_after_profit = vcontract.get_position(&position_id).unwrap();
    // Position price increased to $4.86, asset price = $6
    assert_eq!(position_after_profit.average_price.0, 4864864);

    update_near_price(&mut vcontract, dollars(5));

    // Set size delta = 0
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(0)),
        is_long: true,
        referrer_id: None,
    });

    let position = vcontract.get_position(&position_id).unwrap();
    // Price remains the same, asset price = $5
    assert_eq!(
        position_after_profit.average_price.0,
        position.average_price.0
    );
}

#[test]
fn test_open_short_position_average_price() {
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

    let position_initial = vcontract.get_position(&position_id).unwrap();
    assert_eq!(dollars(5), position_initial.average_price.0);

    update_near_price(&mut vcontract, dollars(4));

    // Set size delta > 0
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    let position_after_loss = vcontract.get_position(&position_id).unwrap();
    // Position price changes downwards $4.44, asset price = $4
    assert_eq!(position_after_loss.average_price.0, 4444444);

    // Update price to $5 back
    update_near_price(&mut vcontract, dollars(5));

    // Set size delta = 0
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(0)),
        is_long: false,
        referrer_id: None,
    });

    let position = vcontract.get_position(&position_id).unwrap();
    // Price remains the same = 4.44$, asset price = 5$
    assert_eq!(
        position_after_loss.average_price.0,
        position.average_price.0
    );

    // Update price to $6
    update_near_price(&mut vcontract, dollars(6));

    // Set size delta > 0
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(50)),
        is_long: false,
        referrer_id: None,
    });

    let position_after_profit = vcontract.get_position(&position_id).unwrap();
    // Position price increased to $4.55, asset price = $6
    assert_eq!(position_after_profit.average_price.0, 4687499);

    update_near_price(&mut vcontract, dollars(4));

    // Set size delta = 0
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(0)),
        is_long: false,
        referrer_id: None,
    });

    let position = vcontract.get_position(&position_id).unwrap();
    // Price remains the same, asset price = $4
    assert_eq!(
        position_after_profit.average_price.0,
        position.average_price.0
    );
}

#[test]
fn test_open_short_position_custom_stable() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.add_asset("usdt".to_string(), 6, true, 50);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft("usdt".parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));
    vcontract.update_index_price(vec![UpdateIndexPriceRequest {
        asset_id: "usdt".to_string(),
        price: U128::from(dollars(1)), // 1 USD
        spread: None,
    }]);

    // Open a 4x leveraged position short NEAR - stable USDT
    // Collateral = $100, size = $200
    set_predecessor_token(&mut context, "usdt".to_string());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(100).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(200).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    let position_initial = vcontract.get_positions(get_account(Alice));
    assert_eq!(position_initial[0].collateral_id, "usdt".to_string());
    assert_eq!(position_initial[0].collateral.0, dollars(100));
    assert_eq!(position_initial[0].size.0, dollars(200));
    assert_eq!(position_initial[0].average_price.0, dollars(5));

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, dollars(0));

    // Set size delta $100, collateral = 10
    set_predecessor_token(&mut context, "usdt".to_string());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(200).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    let position_second_increase = vcontract.get_positions(get_account(Alice));
    assert_eq!(
        position_second_increase[0].collateral_id,
        "usdt".to_string()
    );
    assert_eq!(position_second_increase[0].collateral.0, dollars(110));
    assert_eq!(position_second_increase[0].size.0, dollars(400));
}

#[test]
#[should_panic(expected = "Position leverage is lower than minimum leverage")]
fn test_increase_position_below_min_leverage() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(100));

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(500)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, near(1));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(0)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Leverage positions are currently disabled")]
fn test_open_position_leverage_disabled() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 5x leveraged position long NEAR
    set_deposit(&mut context, near(5));
    // println!("deposit: {}", near_deposit);

    vcontract.set_leverage_enabled(false);
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
fn test_open_with_swap() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    let test_token = "test-token";

    // Add 100 Test token ($10 each) to the pool
    vcontract.add_asset(test_token.into(), 6, false, 25);
    vcontract.contract_mut().add_liquidity(
        &AssetId::Ft(AccountId::new_unchecked(test_token.into())),
        100000000,
    );

    update_asset_price(&mut vcontract, test_token.into(), dollars(10));

    // 10 NEAR = 5 Test Token = 50 USD
    // We expect 2x leverage after the swap
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: test_token.into(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let asset = vcontract.get_asset_info(test_token.into());
    // 1000 - 100 for reserves - 50 from swap + 50 for collateral
    // 1 test token = 10 USD
    assert_eq!(asset.available_liquidity.0, dollars(900) / 10);

    let position = vcontract.get_position(&position_id).unwrap();
    assert_eq!(position.collateral.0, dollars(50));
}

#[test]
#[should_panic(expected = "Contract is temporary paused")]
fn test_open_with_transfer_paused_contract() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(1000),
            short: dollars(1000),
        },
    );

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1010));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Admin);
    vcontract.set_state(ContractState::Paused);
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(999).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(999).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );
}

#[test]
#[should_panic(expected = "Position leverage is higher than maximum leverage")]
fn test_too_high_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 5x leveraged position long NEAR
    set_deposit(&mut context, near(5));

    // 5 nears * 5 dollars = 25 dollars * max leverage 11 = $275 (+1 so that we exceed max leverage)
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(276)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Position leverage is lower than minimum leverage")]
fn test_too_low_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 5x leveraged position long NEAR
    set_deposit(&mut context, near(5));

    // 5 nears * 5 dollars = 25 dollars (-1 so that we are below min leverage of 1)
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(24)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Too much open interest for longs")]
fn test_too_much_oi_longs() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(1000),
            short: dollars(1000),
        },
    );

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(10000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(150));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(999)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(1));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(5)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Too much open interest for shorts")]
fn test_too_much_oi_shorts() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(1000),
            short: dollars(1000),
        },
    );

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1010));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(999).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(999).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(2).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(2).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );
}

#[test]
#[should_panic(expected = "Position size is higher than maximum position size")]
fn test_exceed_long_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                min: 0,
                max: dollars(100),
            },
            short: Limits {
                min: 0,
                max: dollars(100),
            },
        },
    );

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(500));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(50));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: dollars(500).into(),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Position size is lower than minimum position size")]
fn test_below_long_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                min: dollars(10),
                max: dollars(100),
            },
            short: Limits {
                min: 0,
                max: dollars(100),
            },
        },
    );

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(1));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: dollars(5).into(),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic(expected = "Position size is higher than maximum position size")]
fn test_exceed_short_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                min: 0,
                max: dollars(100),
            },
            short: Limits {
                min: 0,
                max: dollars(100),
            },
        },
    );

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(99).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(101).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );
}

#[test]
#[should_panic(expected = "Position size is lower than minimum position size")]
fn test_below_short_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                min: 0,
                max: dollars(100),
            },
            short: Limits {
                min: dollars(10),
                max: dollars(100),
            },
        },
    );

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(1).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(5).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );
}

#[test]
#[should_panic]
fn test_reduce_only_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(5));
    vcontract.set_asset_state(
        near_id(),
        AssetState {
            perps: PerpsState::ReduceOnly,
            lp_support: LpSupportState::Disabled,
            swap: SwapState::Disabled,
        },
    );

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
#[should_panic]
fn test_disabled_perps_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(5));
    vcontract.disable_asset(near_id());

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
fn test_dynamic_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 100,
    });
    vcontract.set_dynamic_position_fees(true);

    update_near_price(&mut vcontract, dollars(5));
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    println!("test");

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_owned()), dollars(1000));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    assert_eq!(
        vcontract
            .get_positions(get_account(Alice))
            .get(0)
            .unwrap()
            .collateral
            .0,
        490 * DOLLAR_DENOMINATION / 10
    );

    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    assert_eq!(
        vcontract.get_position(&position_id).unwrap().collateral.0,
        495 * DOLLAR_DENOMINATION / 10
    );
}

#[test]
#[should_panic(expected = "Not enough reserve to allow the short position")]
fn test_short_more_than_in_stable_balance() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));
    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(100));

    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(101).into(),
        is_long: false,
        referrer_id: None,
    });
}

#[test]
fn test_check_open_interests() {
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
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(100),
            short: dollars(200),
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
#[should_panic(expected = "Too much open interest for shorts")]
fn test_exceed_open_interests() {
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
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(100),
            short: dollars(200),
        },
    );

    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(101)),
        is_long: false,
        referrer_id: None,
    });
}
