mod common;

use common::*;

#[test]
fn test_set_limits() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Admin);
    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(100),
            short: dollars(42),
        },
    );

    let limits = vcontract.get_open_interest_limits(near_id());

    assert_eq!(dollars(100), limits.long);
    assert_eq!(dollars(42), limits.short);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_set_limits_by_non_admin() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    vcontract.set_open_interest_limits(
        near_id(),
        OpenInterestLimits {
            long: dollars(100),
            short: dollars(42),
        },
    );
}
