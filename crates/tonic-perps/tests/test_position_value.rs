mod common;

use common::*;

#[test]
fn test_long_value() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(4));

    // Open position with 2.5x leverage
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: true,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(5));

    let (has_profit, delta) = vcontract.get_position_value(&position_id);

    assert!(has_profit, "Position should be profitable");
    assert_eq!(delta.0, dollars(25), "Incorrect position value");

    update_near_price(&mut vcontract, dollars(4));
    let (_, delta) = vcontract.get_position_value(&position_id);
    assert_eq!(delta.0, 0, "Incorrect position value");

    update_near_price(&mut vcontract, dollars(3));
    let (has_profit, delta) = vcontract.get_position_value(&position_id);
    assert!(!has_profit, "Position should have a loss");
    assert_eq!(delta.0, dollars(25), "Incorrect position value");
}

#[test]
fn test_short_value() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to USDC
    vcontract
        .contract_mut()
        .add_liquidity(&usdc_id().into(), dollars(1000));

    update_near_price(&mut vcontract, dollars(4));

    // Open position with 2.5x leverage
    set_deposit(&mut context, near(10));
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(100)),
        is_long: false,
        referrer_id: None,
    });

    update_near_price(&mut vcontract, dollars(5));

    let (has_profit, delta) = vcontract.get_position_value(&position_id);

    assert!(!has_profit, "Position should not have a profit");
    assert_eq!(delta.0, dollars(25), "Incorrect position value");

    update_near_price(&mut vcontract, dollars(4));
    let (_, delta) = vcontract.get_position_value(&position_id);
    assert_eq!(delta.0, 0, "Incorrect position value");

    update_near_price(&mut vcontract, dollars(3));
    let (has_profit, delta) = vcontract.get_position_value(&position_id);
    assert!(has_profit, "Position should be profitable");
    assert_eq!(delta.0, dollars(25), "Incorrect position value");
}
