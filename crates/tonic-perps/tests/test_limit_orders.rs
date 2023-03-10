mod common;

use common::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::env;

#[test]
fn test_limit_orders_get() {
    let mut limit_orders = LimitOrders::new();
    let mut seq = 0;
    let mut get_next = |limit_order: &LimitOrder| {
        seq += 1;
        return LimitOrderId::new(limit_order, seq);
    };

    let order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    let id = LimitOrderId::new(&order, 1);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let res = limit_orders.get_by_id(&id).unwrap();
    assert_eq!(res.size_delta, order.size_delta);
    assert_eq!(res.owner, order.owner);
    assert_eq!(res.price, order.price);
    assert_eq!(res.is_long, order.is_long);
    assert_eq!(res.collateral_delta, order.collateral_delta);
    assert_eq!(res.collateral_id, order.collateral_id);
    assert_eq!(res.underlying_id, order.underlying_id);
}

#[test]
fn test_limit_orders_get_range() {
    let mut limit_orders = LimitOrders::new();
    let mut seq = 0;
    let mut get_next = |limit_order: &LimitOrder| {
        seq += 1;
        return LimitOrderId::new(limit_order, seq);
    };

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(4),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(6),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    assert_eq!(
        limit_orders
            .get_range(dollars(4), dollars(6), true, ThresholdType::Below)
            .count(),
        3
    );
}

#[test]
fn test_limit_orders_get_higher() {
    let mut limit_orders = LimitOrders::new();
    let mut seq = 0;
    let mut get_next = |limit_order: &LimitOrder| {
        seq += 1;
        return LimitOrderId::new(limit_order, seq);
    };

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(4),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(6),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let mut range =
        limit_orders.get_range_higher_than_price(dollars(5), true, ThresholdType::Below);
    assert_eq!(range.clone().count(), 2);
    assert!(range.all(|e| e.1.price >= dollars(5)));
}

#[test]
fn test_limit_orders_get_lower() {
    let mut limit_orders = LimitOrders::new();
    let mut seq = 0;
    let mut get_next = |limit_order: &LimitOrder| {
        seq += 1;
        return LimitOrderId::new(limit_order, seq);
    };

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(4),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let limit_order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::NEAR,
        underlying_id: AssetId::from(usdc_id()),
        price: dollars(6),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    limit_orders.insert(get_next(&limit_order), limit_order);

    let mut range = limit_orders.get_range_lower_than_price(dollars(5), true, ThresholdType::Below);
    assert_eq!(range.clone().count(), 2);
    assert!(range.all(|e| e.1.price <= dollars(5)));
}

#[test]
fn test_limit_orders_execute() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let params = LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    let limit_order_id = vcontract.add_limit_order(params.clone());

    update_near_price(&mut vcontract, dollars(4));
    set_deposit(&mut context, 1);

    vcontract.execute_limit_order(near_id(), limit_order_id);
    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_positions.len(), 1);
    let position = user_positions.get(0).unwrap();
    assert_eq!(position.size.0, params.size_delta.0);
    assert_eq!(position.collateral_id, near_id());
    assert_eq!(position.collateral.0, dollars(40));
    assert_eq!(position.underlying_id, params.underlying_id);
    assert_eq!(position.is_long, params.is_long);
}

#[test]
#[should_panic(expected = "Position already has a limit order of such type")]
fn test_limit_orders_same_order_type() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let params = LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    vcontract.add_limit_order(params.clone());

    let params = LimitOrderParameters {
        price: dollars(3).into(),
        size_delta: dollars(100).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    vcontract.add_limit_order(params.clone());
}

#[test]
fn test_limit_orders_same_order_type_different_threshold() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let params = LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    vcontract.add_limit_order(params.clone());

    let params = LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(100).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    vcontract.add_limit_order(params.clone());
}

#[test]
#[should_panic(expected = "Position is not ready to be executed")]
fn test_limit_orders_execute_not_ready_below() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let params = LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    let limit_order_id = vcontract.add_limit_order(params.clone());

    set_deposit(&mut context, 1);

    vcontract.execute_limit_order(near_id(), limit_order_id);
}

#[test]
#[should_panic(expected = "Position is not ready to be executed")]
fn test_limit_orders_execute_not_ready_above() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let params = LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(60).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    };
    let limit_order_id = vcontract.add_limit_order(params.clone());

    set_deposit(&mut context, 1);

    vcontract.execute_limit_order(near_id(), limit_order_id);
}

#[test]
fn test_limit_orders_remove_outdated() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let limit_order_id = vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    context.block_timestamp(u64::MAX);
    testing_env!(context.build());

    set_deposit(&mut context, 1);

    vcontract.remove_outdated_limit_order(near_id(), limit_order_id);
    let limit_orders = vcontract.get_limit_orders(near_id());
    assert_eq!(limit_orders.len(), 0);
}

#[test]
#[should_panic(expected = "Limit order has not reached max limit order lifetime")]
fn test_limit_orders_remove_not_outdated() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let limit_order_id = vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, 1);

    vcontract.remove_outdated_limit_order(near_id(), limit_order_id);
}

#[test]
fn test_remove_limit_orders() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let limit_order_id = vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, 1);
    vcontract.remove_limit_order(limit_order_id);
    let limit_orders = vcontract.get_limit_orders(near_id());
    assert_eq!(limit_orders.len(), 0);
}

#[test]
#[should_panic(expected = "You do not have any orders")]
fn test_remove_another_limit_orders() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let limit_order_id = vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);

    vcontract.remove_limit_order(limit_order_id);
}

#[test]
#[should_panic(expected = "You do not have any order with this ID")]
fn test_remove_another_limit_orders_2() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    let limit_order_id = vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_predecessor(&mut context, Alice);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, 1);
    vcontract.remove_limit_order(limit_order_id);
}

#[test]
fn test_limit_orders_ft_transfer() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(4).into(),
            size_delta: dollars(40).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    let limit_orders = vcontract.get_limit_orders(near_id());
    let res = limit_orders.get(0).unwrap();

    assert_eq!(res.size_delta, dollars(40).into());
    assert_eq!(res.owner, get_account(Alice).to_string());
    assert_eq!(res.price, dollars(4).into());
    assert_eq!(res.is_long, false);
    assert_eq!(res.collateral_delta, 0.into());
    assert_eq!(res.attached_collateral, dollars(10).into());
    assert_eq!(res.collateral_id, AssetId::from(usdc_id()).into_string());
    assert_eq!(res.underlying_id, AssetId::from(near_id()).into_string());
}

#[test]
fn test_limit_orders_merge_buy() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    let limit_orders = vcontract.get_limit_orders(near_id());
    assert_eq!(limit_orders.len(), 1);
    let limit_order = limit_orders.get(0).unwrap();
    assert_eq!(limit_order.size_delta, dollars(80).into());
    assert_eq!(limit_order.attached_collateral, near(20).into());
}

#[test]
fn test_limit_orders_merge_sell() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, 0);

    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(10).into()),
    });

    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(10).into()),
    });

    let limit_orders = vcontract.get_limit_orders(near_id());
    assert_eq!(limit_orders.len(), 1);
    let limit_order = limit_orders.get(0).unwrap();
    assert_eq!(limit_order.size_delta, dollars(80).into());
    assert_eq!(limit_order.attached_collateral, 0.into());
    assert_eq!(limit_order.collateral_delta, dollars(20).into());
}

#[test]
#[should_panic(expected = "Can't create a limit order that changes nothing")]
fn test_limit_order_with_no_changes() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    // Both fields are zero
    set_deposit(&mut context, 0);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(0).into()),
    });
}

#[test]
fn test_limit_orders_zero_deltas() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    // Zero size delta
    set_deposit(&mut context, near(5));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    // Zero collateral delta
    set_deposit(&mut context, 0);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(100).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });
}

#[test]
fn test_limit_orders_get_eligible() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });
    set_predecessor(&mut context, Alice);
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    // Add long increase/decrease positions that
    // will be eligible (total: 4, after merge: 2)
    set_predecessor(&mut context, Admin);
    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, 0);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });

    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, 0);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });

    // Add two positions with above/below threshold type
    // (only one eligible)
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 0);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(3).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });

    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));

    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    assert_eq!(eligible_limit_orders.len(), 3);
}

#[test]
fn test_update_limit_orders_after_decrease() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(0));

    // Order to close position if price goes down. Execute this one.
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });

    // Order to increase position if price goes up.
    // Should be removed as to leverage error
    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    // Order to create short position. Should remain.
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Admin),
        dollars(100).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(3).into(),
            size_delta: dollars(200).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 3);

    set_deposit(&mut context, 1);
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 1);
}

#[test]
fn test_update_limit_orders_after_increase() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // Order to increase position if price goes up. Execute this one
    set_deposit(&mut context, near(5));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(600).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_deposit(&mut context, near(0));

    // Order to close position if price goes down.
    // This one should be removed as losses exceed collateral if it is executed.
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(100).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(10).into()),
    });

    // Order to create short position. Should remain.
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Admin),
        dollars(100).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(3).into(),
            size_delta: dollars(200).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(6));
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 3);

    set_deposit(&mut context, 1);

    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 1);
}

#[test]
fn test_update_limit_orders_after_liquidate() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(0));

    // Order to close position if price goes down. Execute this one.
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });

    // Order to increase position if price goes up.
    // Should be removed as to leverage error
    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    // Order to create short position. Should remain.
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Admin),
        dollars(100).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(3).into(),
            size_delta: dollars(200).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(1));
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 3);

    let status = vcontract.get_liquidation_status(&position_id);
    assert!(status.insolvent);
    assert!(!status.max_leverage_exceeded);

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);

    vcontract.liquidate_position(LiquidatePositionRequest { position_id });

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 1);
}

#[test]
fn test_add_decrease_limit_order_decrease_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    vcontract.set_limit_orders_state(LimitOrdersState::DecreaseOnly);
    set_deposit(&mut context, near(0));

    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(45) / 10).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });
}

#[test]
#[should_panic(expected = "Limit orders are disable or only decrease ones are allowed")]
fn test_add_increase_limit_order_decrease_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_limit_orders_state(LimitOrdersState::DecreaseOnly);

    set_deposit(&mut context, near(10));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });
}

#[test]
#[should_panic(expected = "Limit orders are disabled")]
fn test_add_decrease_limit_order_disabled_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, near(30));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(300).into(),
        is_long: true,
        referrer_id: None,
    });

    vcontract.set_limit_orders_state(LimitOrdersState::Disabled);
    set_deposit(&mut context, near(0));

    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(45) / 10).into(),
        size_delta: dollars(40).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });
}

#[test]
#[should_panic(expected = "Limit orders are disable or only decrease ones are allowed")]
fn test_ft_transfer_order_in_decrease_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_limit_orders_state(LimitOrdersState::DecreaseOnly);

    set_deposit(&mut context, near(10));
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(4).into(),
            size_delta: dollars(40).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: true,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );
}

#[test]
fn test_is_eligible() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    let mut order = LimitOrder {
        owner: get_account(Alice),
        attached_collateral: 0,
        collateral_delta: near(100),
        size_delta: dollars(1000),
        collateral_id: AssetId::from(usdc_id()),
        underlying_id: AssetId::NEAR,
        price: dollars(5),
        is_long: true,
        expiry: env::block_timestamp_ms() + 1000,
        order_type: OrderType::Increase,
        threshold: ThresholdType::Below,
    };
    assert!(vcontract.contract_mut().limit_order_is_eligible(&order));
    order.price = dollars(4);
    assert!(!vcontract.contract_mut().limit_order_is_eligible(&order));
    order.price = dollars(6);
    assert!(vcontract.contract_mut().limit_order_is_eligible(&order));
    order.threshold = ThresholdType::Above;
    assert!(!vcontract.contract_mut().limit_order_is_eligible(&order));
    order.price = dollars(4);
    assert!(vcontract.contract_mut().limit_order_is_eligible(&order));
}

#[test]
fn test_remove_invalid_limit_order_losses_exceed() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // Order to close the position in case price drop
    set_deposit(&mut context, near(0));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 1);

    update_near_price(&mut vcontract, dollars(45) / 10);

    // Increase position again.
    set_deposit(&mut context, near(2));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // User's loss on the price he specified in limit order would be:
    // (collateral $400 - limit order size delta $200) * ($4.736 - $4) / $4.736 = $31.1.
    // His collateral is $50 (first increase) + $9 (second increase) -
    // - $50 (limit order collateral delta) = $9.
    // Remaining collateral is $9 while loss is $31.1, this order couldn't be executed as
    // losses would exceed collateral, remove it.
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 0);
}

#[test]
fn test_remove_invalid_limit_order_max_leverage_exceed() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // Order to close the position in case price drop
    set_deposit(&mut context, near(0));
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(4).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 1);

    update_near_price(&mut vcontract, dollars(45) / 10);

    // Increase position again.
    set_deposit(&mut context, near(10));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // User's loss on the price he specified in limit order would be:
    // (collateral $400 - limit order size delta $200) * ($4.736 - $4) / $4.736 = $31.1.
    // His collateral is $50 (first increase) + $45 (second increase) -
    // - $50 (limit order collateral delta) = $45.
    // Remaining collateral is $45 - $31.1 = $13.9, size is $200, leverage is 14.3x.
    // The execution of this limit order would cause MaxLeverageExceed error, remove it.
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 0);
}

#[test]
#[should_panic(expected = "Losses will exceed collateral")]
fn test_order_to_close_position_with_huge_losses() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(0));

    // Try to add a close position order with a price when losses exceed collateral.
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(3).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });
}

#[test]
#[should_panic(expected = "Losses will exceed remaining collateral")]
fn test_order_to_decrease_position_with_huge_losses() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    set_deposit(&mut context, near(0));

    // Try to add a decrease position order with a price when losses exceed collateral.
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(3).into(),
        size_delta: dollars(100).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(20).into()),
    });
}

#[test]
fn test_do_not_merge_different_limit_orders() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));

    set_deposit(&mut context, 1);
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(4).into(),
            size_delta: dollars(40).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    vcontract.add_asset("usdt".to_string(), 6, true, 50);
    update_asset_price(&mut vcontract, "usdt".to_string(), dollars(1));

    set_predecessor_token(&mut context, "usdt".to_string());

    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(4).into(),
            size_delta: dollars(40).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: false,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );

    let limit_orders = vcontract.get_limit_orders(near_id());
    assert_eq!(limit_orders.len(), 2);
}

#[test]
#[should_panic(
    expected = "Collateral token must equal underlying to create limit order for long position"
)]
fn test_stable_token_for_long_position() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, 1);
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(10).into(),
        serde_json::to_string(&Action::PlaceLimitOrder(LimitOrderParameters {
            price: dollars(4).into(),
            size_delta: dollars(40).into(),
            underlying_id: near_id(),
            collateral_id: None,
            is_long: true,
            expiry: None,
            order_type: OrderType::Increase,
            collateral_delta: None,
        }))
        .unwrap(),
    );
}

#[test]
fn test_increase_with_zero_collateral_after_closure() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // Add two eligible orders. Execution of one order makes the other one invalid.

    // This order should close the position. Next increase order increases only size
    // delta so it becomes invalid
    set_deposit(&mut context, near(0));
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(45) / 10).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(dollars(50).into()),
    });

    // This one should increase size delta, so limit order to close the position would
    // be insolvent.
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(55) / 10).into(),
        size_delta: dollars(10).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 2);

    set_deposit(&mut context, 1);
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(user_orders.len(), 0);
}

#[test]
fn test_close_order_after_increase_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // This order should close the position. Next increase order increases only collateral
    // so anyway order should remain as size delta equals.
    set_deposit(&mut context, near(0));
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(45) / 10).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: None,
    });

    set_deposit(&mut context, near(2));
    // This one should increase collateral
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(55) / 10).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(55) / 10);
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 2);

    set_deposit(&mut context, 1);
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    update_near_price(&mut vcontract, dollars(45) / 10);
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 1);

    // Executu limit order to close the position.
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_orders.len(), 0);
    assert_eq!(user_positions.len(), 0);
}

#[test]
fn test_close_order_after_decrease_collateral() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    // This order should close the position. Next order decreases only collateral
    // so anyway order should remain as size delta equals.
    set_deposit(&mut context, near(0));
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(45) / 10).into(),
        size_delta: dollars(200).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: None,
    });

    // This one should decrease collateral
    vcontract.add_limit_order(LimitOrderParameters {
        price: (dollars(55) / 10).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: Some(near_id()),
        is_long: true,
        expiry: None,
        order_type: OrderType::Decrease,
        collateral_delta: Some(U128(10)),
    });

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(55) / 10);
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 2);

    set_deposit(&mut context, 1);
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    update_near_price(&mut vcontract, dollars(45) / 10);
    let eligible_limit_orders = vcontract.get_eligible_orders(near_id(), None);
    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    assert_eq!(eligible_limit_orders.len(), 1);
    assert_eq!(user_orders.len(), 1);

    // Executu limit order to close the position.
    vcontract.execute_limit_order(near_id(), eligible_limit_orders[0]);

    let user_orders = vcontract.get_user_limit_orders(&get_account(Admin));
    let user_positions = vcontract.get_positions(get_account(Admin));
    assert_eq!(user_orders.len(), 0);
    assert_eq!(user_positions.len(), 0);
}

#[test]
fn test_get_collateral_after_removing_order_in_increase() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(110)),
        is_long: true,
        referrer_id: None,
    });

    let order_attached_amount = near(10);
    set_deposit(&mut context, order_attached_amount);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(6).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    let (_, transfer_info) = vcontract.contract_mut().increase_position(
        &get_account(Admin),
        &AssetId::from(near_id()),
        &AssetId::from(near_id()),
        near(5),
        dollars(10),
        true,
        None,
    );

    assert_eq!(transfer_info.amount(), order_attached_amount);
    assert_eq!(transfer_info.receiver_id(), get_account(Admin));
    assert_eq!(transfer_info.asset_id(), AssetId::from(near_id()));
}

#[test]
fn test_get_collateral_after_removing_order_in_decrease() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    // add liquidity to NEAR
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(1000));

    update_near_price(&mut vcontract, dollars(5));
    set_deposit(&mut context, near(10));

    // Open a 4x leveraged position long NEAR
    let position_id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: "near".to_string(),
        size_delta: U128(dollars(200)),
        is_long: true,
        referrer_id: None,
    });

    let order_attached_amount = near(10);
    set_deposit(&mut context, order_attached_amount);
    vcontract.add_limit_order(LimitOrderParameters {
        price: dollars(7).into(),
        size_delta: dollars(0).into(),
        underlying_id: near_id(),
        collateral_id: None,
        is_long: true,
        expiry: None,
        order_type: OrderType::Increase,
        collateral_delta: None,
    });

    update_near_price(&mut vcontract, dollars(6));
    let transfer_info = vcontract.contract_mut().decrease_position(
        position_id,
        dollars(10),
        dollars(100),
        None,
        false,
        None,
    );

    // Collateral delta $10 + $20 of profit = 5 NEAR
    let profit = near(5);
    assert_eq!(transfer_info.amount(), order_attached_amount + profit);
    assert_eq!(transfer_info.receiver_id(), get_account(Admin));
    assert_eq!(transfer_info.asset_id(), AssetId::from(near_id()));
}
