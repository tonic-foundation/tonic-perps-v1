mod common;

use common::*;

use near_contract_standards::fungible_token::core::FungibleTokenCore;

#[test]
#[should_panic(expected = "Exceeded withdrawal limit")]
fn test_exceed_withdrawal_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // Sliding window of 1s and 50% withdrawal limit
    vcontract.set_withdrawal_limits_settings(
        near_id(),
        Some(std::time::Duration::from_secs(1).as_millis() as u64),
        Some(U128(5000)),
    );

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    vcontract.burn_lp_token(U128(lp_tokens(249.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(251.));

    vcontract.burn_lp_token(U128(lp_tokens(2.)), near_id(), None, None);
}

#[test]
fn test_withdrawal_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // Sliding window of 1ms and 50% withdrawal limit
    vcontract.set_withdrawal_limits_settings(
        near_id(),
        Some(std::time::Duration::from_secs(1).as_millis() as u64),
        Some(U128(5000)),
    );

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);
    vcontract.burn_lp_token(U128(lp_tokens(249.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(251.));

    context.block_timestamp(
        near_sdk::env::block_timestamp() + std::time::Duration::from_secs(2).as_nanos() as u64,
    );
    testing_env!(context.build());

    // Wait for previous withdrawal to get out of the sliding window
    vcontract.burn_lp_token(U128(lp_tokens(2.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(249.));
}

#[test]
fn test_deposit_withdrawal_limit() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    // Sliding window of 1s and 50% withdrawal limit
    vcontract.set_withdrawal_limits_settings(
        near_id(),
        Some(std::time::Duration::from_secs(1).as_millis() as u64),
        Some(U128(5000)),
    );

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    vcontract.burn_lp_token(U128(lp_tokens(249.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(251.));

    set_predecessor(&mut context, Alice);
    set_deposit(&mut context, near(100));
    vcontract.mint_lp_near(None, None);

    vcontract.burn_lp_token(U128(lp_tokens(2.)), near_id(), None, None);

    let balance = vcontract.ft_balance_of(get_account(Alice)).0;
    assert_eq!(balance, lp_tokens(749.));
}
