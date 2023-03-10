mod common;

use common::*;

#[test]
fn test_add_admin() {
    let (mut _context, mut vcontract) = setup();
    vcontract.add_admin(get_account(Alice), AdminRole::FullAdmin);
    assert!(vcontract.is_admin(get_account(Alice)));
}

#[test]
#[should_panic(expected = "caller must be owner")]
fn test_unauthorized_add_admin() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.add_admin(get_account(Alice), AdminRole::FullAdmin);
}

#[test]
fn test_remove_admin() {
    let (mut _context, mut vcontract) = setup();
    vcontract.add_admin(get_account(Alice), AdminRole::FullAdmin);
    vcontract.remove_admin(get_account(Alice));
    assert!(!vcontract.is_admin(get_account(Alice)));
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_remove_admin() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.remove_admin(get_account(Admin));
}

#[test]
fn test_add_price_oracle() {
    let (mut context, mut vcontract) = setup();
    vcontract.add_price_oracle(get_account(Alice));
    set_predecessor(&mut context, Alice);
    update_near_price(&mut vcontract, 42);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_add_price_oracle() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.add_price_oracle(get_account(Alice));
}

#[test]
#[should_panic(expected = "caller must be approved price oracle")]
fn test_remove_price_oracle() {
    let (mut context, mut vcontract) = setup();
    vcontract.add_price_oracle(get_account(Alice));
    vcontract.remove_price_oracle(get_account(Alice));
    set_predecessor(&mut context, Alice);
    update_near_price(&mut vcontract, 42);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_remove_price_oracle() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.remove_price_oracle(get_account(Admin));
}

#[test]
#[should_panic(expected = "caller must be owner")]
fn test_set_owner() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_owner(get_account(Alice));

    // Admin is not the owner anymore so this should panic
    vcontract.set_owner(get_account(Admin));
}

#[test]
#[should_panic(expected = "caller must be owner")]
fn test_unauthorized_set_owner() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_owner(get_account(Alice));
}

#[test]
fn test_set_state() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_state(ContractState::Paused);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_state(ContractState::Paused);
}

#[test]
fn test_set_funding_interval() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_funding_interval(10);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_funding_interval() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_funding_interval(10);
}

#[test]
fn test_set_fee_parameters() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 1,
        stable_tax_bps: 2,
        mint_burn_fee_bps: 3,
        swap_fee_bps: 4,
        stable_swap_fee_bps: 5,
        margin_fee_bps: 6,
    });
    let fees = vcontract.get_fee_parameters();

    assert_eq!(fees.tax_bps, 1);
    assert_eq!(fees.stable_tax_bps, 2);
    assert_eq!(fees.mint_burn_fee_bps, 3);
    assert_eq!(fees.swap_fee_bps, 4);
    assert_eq!(fees.stable_swap_fee_bps, 5);
    assert_eq!(fees.margin_fee_bps, 6);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_fee_parameters() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 1,
        stable_tax_bps: 2,
        mint_burn_fee_bps: 3,
        swap_fee_bps: 4,
        stable_swap_fee_bps: 5,
        margin_fee_bps: 6,
    });
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unathorized_add_asset() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.add_asset("dodgecoin".to_string(), 42, false, 42);
}

#[test]
fn test_set_shortable() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_shortable("near".to_string(), true);
    let assets = vcontract.get_assets();
    let near = assets.iter().find(|a| a.id == "near").unwrap();
    assert!(near.shortable);

    vcontract.set_shortable("near".to_string(), false);
    let assets = vcontract.get_assets();
    let near = assets.iter().find(|a| a.id == "near").unwrap();
    assert!(!near.shortable);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_shortable() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_shortable("near".to_string(), true);
}

#[test]
fn test_set_goblins() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_goblins(vec![]);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_goblins() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_goblins(vec![]);
}

#[test]
fn test_add_goblins() {
    let (mut _context, mut vcontract) = setup();
    vcontract.add_goblins(vec![]);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_add_goblins() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.add_goblins(vec![]);
}

#[test]
fn test_remove_goblins() {
    let (mut _context, mut vcontract) = setup();
    vcontract.remove_goblins(vec![]);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_remove_goblins() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.remove_goblins(vec![]);
}

#[test]
fn test_set_switchboard_aggregator_address() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_switchboard_aggregator_address(near_id(), Some([0; 32]));
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_switchboard_aggregator_address() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_switchboard_aggregator_address(near_id(), Some([0; 32]));
}

#[test]
fn test_set_max_asset_price_change() {
    let (mut context, mut vcontract) = setup();
    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_max_asset_price_change(near_id(), Some(U128(100)));
    context.block_timestamp(
        context.context.block_timestamp + std::time::Duration::from_secs(1).as_nanos() as u64,
    );
    testing_env!(context.build());
    update_near_price(&mut vcontract, dollars(10));
    let assets = vcontract.get_assets();
    let near = assets.iter().find(|a| a.id == "near").unwrap();
    assert_eq!(near.average_price.0, 5050000)
}

#[test]
fn test_set_max_asset_price_change_zero() {
    let (mut _context, mut vcontract) = setup();
    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_max_asset_price_change(near_id(), Some(U128(0)));
    update_near_price(&mut vcontract, dollars(10));
    let assets = vcontract.get_assets();
    let near = assets.iter().find(|a| a.id == "near").unwrap();
    assert_eq!(near.average_price.0, dollars(5))
}

#[test]
fn test_set_max_asset_price_change_none() {
    let (mut _context, mut vcontract) = setup();
    update_near_price(&mut vcontract, dollars(5));
    vcontract.set_max_asset_price_change(near_id(), None);
    update_near_price(&mut vcontract, dollars(10));
    let assets = vcontract.get_assets();
    let near = assets.iter().find(|a| a.id == "near").unwrap();
    assert_eq!(near.average_price.0, dollars(10))
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_max_asset_price_change() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_max_asset_price_change(near_id(), Some(U128(10)));
}

#[test]
fn test_set_open_interest_limits() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: 10,
            short: 10,
        },
    );
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_open_interest_limits() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: 10,
            short: 10,
        },
    );
}

#[test]
fn test_set_position_limits() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_position_limits(near_id(), Default::default());
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_position_limits() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_position_limits(near_id(), Default::default());
}

#[test]
fn test_set_dynamic_fees() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_dynamic_swap_fees(true);
    vcontract.set_dynamic_position_fees(false);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_dynamic_swap_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_dynamic_swap_fees(true);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_dynamic_position_fees() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_dynamic_position_fees(false);
}

#[test]
fn test_set_withdrawal_limits() {
    let (mut _context, mut vcontract) = setup();
    vcontract.set_withdrawal_limits_settings(near_id(), Some(42), Some(U128(42)));
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_unauthorized_set_withdrawal_limits() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Alice);
    vcontract.set_withdrawal_limits_settings(near_id(), Some(42), Some(U128(42)));
}

#[test]
fn test_switch_asset_state() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    vcontract.disable_asset(near_id());
    let asset_state = vcontract.get_asset_state(near_id());
    assert_eq!(asset_state.perps, PerpsState::Disabled);
    assert_eq!(asset_state.lp_support, LpSupportState::Disabled);
    assert_eq!(asset_state.swap, SwapState::Disabled);

    vcontract.enable_asset(near_id());
    let asset_state = vcontract.get_asset_state(near_id());
    assert_eq!(asset_state.perps, PerpsState::Enabled);
    assert_eq!(asset_state.lp_support, LpSupportState::Enabled);
    assert_eq!(asset_state.swap, SwapState::Enabled);
}

#[test]
fn test_set_asset_weight() {
    let (mut _context, mut vcontract) = setup();
    let asset_before = vcontract.get_asset_info(near_id());
    let total_weights = vcontract.get_total_token_weights();
    assert_eq!(asset_before.token_weight, 50);
    assert_eq!(total_weights, 100);
    vcontract.update_asset_weight(near_id(), 20);
    let asset_after = vcontract.get_asset_info(near_id());
    let total_weights = vcontract.get_total_token_weights();
    assert_eq!(asset_after.token_weight, 20);
    assert_eq!(total_weights, 70);
}
