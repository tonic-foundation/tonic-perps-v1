mod common;

use common::*;

#[test]
fn test_long_position_liquidation_price() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));

    // Open a 4x leveraged position long NEAR
    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_positions[0].liquidation_price.margin_fees, 2750000); // $2.75
    assert_eq!(user_positions[0].liquidation_price.max_leverage, 2954546); // $2.95
}

#[test]
fn test_short_position_liquidation_price() {
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

    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_positions[0].liquidation_price.margin_fees, 7250000); // $7.25
    assert_eq!(user_positions[0].liquidation_price.max_leverage, 7045454); // $7.04
}
