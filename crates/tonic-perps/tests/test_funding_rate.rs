mod common;

use common::*;

#[test]
fn test_funding_rate() {
    // 100 = 0.01%
    let base_funding_rate: u64 = 100;
    let mut asset = Asset::new(usdc_id().into(), 6, true, 25, base_funding_rate);

    asset.update_cumulative_funding_rate(30, 60);

    // Should not update in increments less than 1m
    assert_eq!(asset.cumulative_funding_rate, 0);

    asset.update_cumulative_funding_rate(90, 60);

    // Should not update if there's nothing in the pool
    assert_eq!(asset.cumulative_funding_rate, 0);
    assert_eq!(asset.last_funding_time, 60);
    assert_eq!(asset.current_funding_rate(), asset.min_funding_rate());

    asset.add_liquidity(100, &get_account(Admin));
    asset.update_cumulative_funding_rate(120, 60);
    assert_eq!(
        asset.cumulative_funding_rate,
        asset.min_funding_rate().into(),
        "Asset did not accumulate funding"
    );

    asset.increase_reserved_amount(50, &get_account(Admin));
    assert_eq!(
        asset.current_funding_rate(),
        base_funding_rate / 2,
        "Asset funding rate not accounting for utilization"
    );
}

#[test]
fn test_position_funding() {
    let base_funding_rate: u128 = 100;

    assert_eq!(
        get_funding_fee(dollars(100000), 0, base_funding_rate),
        dollars(10)
    );
}
