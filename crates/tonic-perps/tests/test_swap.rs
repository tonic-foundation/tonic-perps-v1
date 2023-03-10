mod common;

use common::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[test]
fn test_simple_swap() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), 10000000000);

    assert_eq!(
        vcontract
            .contract_mut()
            .swap(&AssetId::NEAR, &AssetId::from(usdc_id()), near(1), None,),
        dollars(5)
    );
}

#[test]
fn test_simple_swap_low_decimals() {
    let (mut context, mut vcontract) = setup();
    vcontract.add_asset("aurora".to_string(), 20, false, 50);

    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract.update_index_price(vec![UpdateIndexPriceRequest {
        asset_id: "aurora".to_string(),
        price: U128::from(300000), // 0.3 USD
        spread: None,
    }]);

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(10));

    // In dollars swap AURORA amount is 0
    assert_eq!(
        vcontract.contract_mut().swap(
            &AssetId::from("aurora".to_string()),
            &AssetId::NEAR,
            5 * 10u128.pow(12), // swap 5 * 10^12 AURORA
            None,
        ),
        3 * 10u128.pow(15) // NEAR
    );
}

#[test]
fn test_simple_swap_with_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_dynamic_swap_fees(false);
    vcontract.set_dynamic_position_fees(false);

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 500,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), 10000000000);

    assert_eq!(
        vcontract
            .contract_mut()
            .swap(&AssetId::NEAR, &AssetId::from(usdc_id()), near(1), None,),
        dollars(475) / 100
    );
}

#[test]
fn test_simple_swap_with_dynamic_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_dynamic_swap_fees(true);
    vcontract.set_dynamic_position_fees(true);

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 500,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 500,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(1000));
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(10));

    assert_eq!(
        vcontract
            .contract_mut()
            .swap(&AssetId::from(usdc_id()), &AssetId::NEAR, dollars(50), None,),
        // Fee calculation is rather complex :
        // S = swap_fee_bps
        // T = tax_bps
        // To = total amount of dollars in asset
        // Toc = total amount of dollars in contract
        // w = weight ratio (asset weight / total weigh)
        // a = amount out
        // S + ((|To - a - Toc * w| + |To - Toc * w|) / 2 * T / (Toc * w)) / 10_000
        near(10) - near(10) * (500 + (500 * 500 / 525)) / 10000
    );
}

#[test]
#[should_panic(expected = "Exceeded slippage tolerance")]
fn test_swap_min_amount_out() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), 10000000000);

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 500,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract.contract_mut().swap(
        &AssetId::NEAR,
        &AssetId::from(usdc_id()),
        near(1),
        Some(dollars(5) + 1),
    );
}

#[test]
#[should_panic(expected = "Not enough liquidity to perform swap")]
fn test_swap_more_than_available() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 500,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });

    vcontract.contract_mut().swap(
        &AssetId::NEAR,
        &AssetId::from(usdc_id()),
        near(1),
        Some(dollars(5)),
    );
}

#[test]
#[should_panic(expected = "Vault: poolAmount < buffer")]
fn test_swap_exceed_buffer_amount() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // Add $10000 of USDC liquidity
    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::from(usdc_id()), dollars(10000));

    // Set buffer amount as $6000
    vcontract.set_buffer_amount(usdc_id(), U128(dollars(6000)));

    // Amount out is $5000, pool balance becomes $5000 < buffer amount $6000
    vcontract
        .contract_mut()
        .swap(&AssetId::NEAR, &AssetId::from(usdc_id()), near(1000), None);
}

// Swaps using ft_on_transfer. Unfortunatelly, no meaningful value is returned so this
// only tests that no panic happens
#[test]
fn test_swap_high_level() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Admin),
        U128(dollars(5)),
        serde_json::ser::to_string(&Action::Swap(SwapParams {
            min_out: None,
            referrer_id: None,
            output_token_id: near_id(),
        }))
        .unwrap(),
    );

    let assets = vcontract.get_assets();
    let near_asset = assets.iter().find(|a| a.id == "near").unwrap();

    assert_eq!(near_asset.pool_amount.0, near(99));
}
