mod common;

use common::*;
use proptest::prelude::*;

use near_contract_standards::fungible_token::{
    core::FungibleTokenCore, receiver::FungibleTokenReceiver,
};

#[test]
fn test_long_aum() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(2000).into(),
        serde_json::to_string(&Action::MintLp(MintLpParams {
            min_out: None,
            referrer_id: None,
        }))
        .unwrap(),
    );

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(7000.0));

    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(5000));

    let usdc_asset = vcontract.get_asset_info(usdc_id());
    assert_eq!(usdc_asset.aum.0, dollars(2000));

    set_deposit(&mut context, near(100));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(1000).into(),
        is_long: true,
        referrer_id: None,
    });

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(10));
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(9500));
}

#[test]
fn test_short_aum() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(10000));

    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(5000));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(500).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(1000).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(3800));
}

#[test]
fn test_short_aum_up() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(10000));

    // Short of 5x
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(400).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(2000).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    // Price goes up by 20%
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(6));

    // Protocol wins 20% of 2000
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(400));
}

#[test]
fn test_short_aum_down() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(10000));

    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    // Short of 5x
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(400).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(2000).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    // Price goes down by 20%
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));

    // Protocol loses 20% of the initial 5000 dollars and 20% of the shorts
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(3600));
}

#[test]
fn test_short_profits_bigger_than_aum() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                max: u128::MAX,
                min: 0,
            },
            short: Limits {
                max: u128::MAX,
                min: 0,
            },
        },
    );

    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(100000));

    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    // Short of 5x
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(4000).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(20000).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));

    // Lose 20% of 5000 and 20% of 40 000 (total of 9000)
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(0));
}

#[test]
fn test_long_aum_up() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    // Long of 2x
    set_deposit(&mut context, near(100));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(1000).into(),
        is_long: true,
        referrer_id: None,
    });

    // Price goes up by 20%
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(6));

    // Protocol loses 20% of 1000 - 500 and wins 20% of 4500
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(5900));
}

#[test]
fn test_long_aum_down() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    // Long of 2x
    set_deposit(&mut context, near(100));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(1000).into(),
        is_long: true,
        referrer_id: None,
    });

    // Price goes down by 20%
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(4));

    // Protocol wins 20% of 1000 - 500 and loses 20% of 4500
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(4100));
}

#[test]
#[should_panic(expected = "Not enough reserve to allow the long position")]
fn test_long_profits_bigger_than_aum() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    // Long of 2x
    set_deposit(&mut context, near(100));
    vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(1000).into(),
        is_long: true,
        referrer_id: None,
    });
}

#[test]
fn test_complex_aum() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                max: u128::MAX,
                min: 0,
            },
            short: Limits {
                max: u128::MAX,
                min: 0,
            },
        },
    );
    vcontract.set_withdrawal_limits_settings(near_id(), Some(0), Some(U128(0)));

    set_predecessor(&mut context, Alice);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from("usdc".to_string()), dollars(100000));

    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    // Long of 2x
    set_deposit(&mut context, near(100));
    let id = vcontract.increase_position(IncreasePositionRequest {
        underlying_id: near_id(),
        size_delta: dollars(1000).into(),
        is_long: true,
        referrer_id: None,
    });

    // Short of 5x
    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        dollars(4000).into(),
        serde_json::to_string(&Action::IncreasePosition(IncreasePositionRequest {
            underlying_id: near_id(),
            size_delta: dollars(20000).into(),
            is_long: false,
            referrer_id: None,
        }))
        .unwrap(),
    );

    // $5000 initial + 100 NEAR - $500 collateral from long
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(5000));

    // Price goes up by 20%
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(6));

    // Protocol loses 20% of 1000 - 500 (so 100), wins 20% of 20000 (so 4000)
    // 1100 NEAR ($6600) - $500 collateral - $200 profit on long + $4000 from short
    // short collateral is under the USDC asset
    let near_asset = vcontract.get_asset_info(near_id());
    assert_eq!(near_asset.aum.0, dollars(9900));

    // Pay out $250 of collateral + $100 of the profits with NEAR at $6
    // $350 / $6 = 58.3333 NEAR
    // Pool should have 1041.666666 NEAR after this
    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, 1);
    vcontract.decrease_position(DecreasePositionRequest {
        size_delta: dollars(500).into(),
        position_id: id,
        collateral_delta: dollars(250).into(),
        referrer_id: None,
        output_token_id: None,
    });
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    let near_asset = vcontract.get_asset_info(near_id());
    // 1041.666666 NEAR * $5 - $250 remaining collateral
    // No PnL now that price dropped back down
    assert_eq!(near_asset.aum.0, 4958333333)
}

proptest! {
    #[test]
    fn test_aum(
        swap_iteration in 1..8,
        swap_amount_near in near(1)..near(500),
        first_price in dollars(1)..dollars(20),
        swap_fee_bps in (1u16..100u16),
    ) {
        let (mut context, mut vcontract) = setup();
        set_predecessor(&mut context, Admin);
        update_near_price(&mut vcontract, first_price);

        // add liquidity to USDC
        vcontract
            .contract_mut()
            .add_liquidity(&AssetId::Ft(usdc_id().parse().unwrap()), dollars(100000));

        set_predecessor(&mut context, Alice);

        set_deposit(&mut context, near(100000));
        vcontract.mint_lp_near(None, None);
        let near_asset_before = vcontract.get_asset_info(near_id());
        let usdc_asset_before = vcontract.get_asset_info(usdc_id());
        let old_aum = near_asset_before.aum.0 + usdc_asset_before.aum.0;

        set_predecessor(&mut context, Admin);

        vcontract.set_fee_parameters(FeeParameters {
            tax_bps: 0,
            stable_tax_bps: 0,
            mint_burn_fee_bps: 0,
            swap_fee_bps: swap_fee_bps,
            stable_swap_fee_bps: swap_fee_bps,
            margin_fee_bps: 0,
        });

        let near_asset_before = vcontract.get_asset_info(near_id());
        let swap_amount_usd = ratio(near_asset_before.average_price.0, swap_amount_near, NEAR_DENOMINATION);

        for _ in 0..=swap_iteration {
            vcontract
                .contract_mut()
                .swap(&usdc_id().into(), &AssetId::NEAR, swap_amount_usd, None);
        }

        update_near_price(&mut vcontract, first_price);

        let near_asset = vcontract.get_asset_info(near_id());
        let usdc_asset = vcontract.get_asset_info(usdc_id());
        let first_aum = near_asset.aum.0 + usdc_asset.aum.0;
        assert_eq!(first_aum, old_aum);

        for _ in 0..=swap_iteration {
            vcontract
                .contract_mut()
                .swap(&AssetId::NEAR, &usdc_id().into(), swap_amount_near, None);
        }

        let near_asset = vcontract.get_asset_info(near_id());
        let usdc_asset = vcontract.get_asset_info(usdc_id());

        let second_aum = near_asset.aum.0 + usdc_asset.aum.0;
        // Convertion from bigger decimals to smaller one gives
        // additional $0.000001 profit
        assert!(second_aum - old_aum <= swap_iteration as u128);

        vcontract.remove_admin(get_account(Admin));
    }
}
