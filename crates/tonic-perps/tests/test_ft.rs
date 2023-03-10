mod common;

use common::*;
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore, receiver::FungibleTokenReceiver,
};

#[test]
fn test_mint_burn_tokens() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    let near_deposit = near(1000);
    let usdc_deposit = dollars(200);

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near_deposit);
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(5000.0));

    set_predecessor_token(&mut context, usdc_id());
    vcontract.ft_on_transfer(
        get_account(Alice),
        usdc_deposit.into(),
        serde_json::to_string(&Action::MintLp(MintLpParams {
            min_out: None,
            referrer_id: None,
        }))
        .unwrap(),
    );

    // $5000 mint for NEAR + $200 mint for USDC
    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(5200.0));

    let near_asset = vcontract.get_asset_info(near_id());
    let usdc_asset = vcontract.get_asset_info(usdc_id());
    assert_eq!(near_deposit, near_asset.pool_amount.0);
    assert_eq!(usdc_deposit, usdc_asset.pool_amount.0);

    set_predecessor(&mut context, Alice);
    let usdc_out = vcontract.burn_lp_token(U128(lp_tokens(200.)), usdc_id(), None, None);
    let near_out = vcontract.burn_lp_token(U128(lp_tokens(5000.)), near_id(), None, None);

    assert_eq!(near_deposit, near_out.0);
    assert_eq!(usdc_deposit, usdc_out.0);

    let near_asset = vcontract.get_asset_info(near_id());
    let usdc_asset = vcontract.get_asset_info(usdc_id());
    assert_eq!(near_asset.pool_amount.0, 0);
    assert_eq!(usdc_asset.pool_amount.0, 0);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(0.0));
}

#[test]
fn test_transfer_tokens() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(5000.0));

    let supply_before = vcontract.ft_total_supply().0;
    assert_eq!(supply_before, lp_tokens(5000.0));

    set_deposit(&mut context, 1);
    vcontract.ft_transfer(get_account(Bob), U128(lp_tokens(5000.0)), None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(0.0));

    let balance = vcontract.ft_balance_of(get_account(Bob)).0;
    assert_eq!(balance, lp_tokens(5000.0));

    let supply_after = vcontract.ft_total_supply().0;
    assert_eq!(supply_before, supply_after);
}

#[test]
fn test_mock_storage_deposit() {
    let (_, mut vcontract) = setup();
    let storage_deposit = vcontract.storage_deposit(Some(get_account(Admin)), None);
    // Default value 125 milliNEAR
    assert_eq!(storage_deposit.total.0, 125_000_000_000_000_000_000_000);
}

#[test]
#[should_panic(expected = "Requires min 0.001GIN to transfer as receiver is not registered")]
fn test_transfer_to_not_registered_account() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(1000));
    vcontract.mint_lp_near(None, None);

    set_deposit(&mut context, 1);
    vcontract.ft_transfer(get_account(Bob), U128(10000000), None);
}
