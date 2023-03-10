mod common;

use common::*;

#[test]
fn test_set_limits() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Admin);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                max: dollars(100),
                min: dollars(0),
            },
            short: Limits {
                max: dollars(42),
                min: dollars(10),
            },
        },
    );

    let limits = vcontract.get_position_limits(near_id());

    assert_eq!(dollars(0), limits.long.min);
    assert_eq!(dollars(100), limits.long.max);
    assert_eq!(dollars(10), limits.short.min);
    assert_eq!(dollars(42), limits.short.max);
}

#[test]
#[should_panic(expected = "caller must be approved admin")]
fn test_set_limits_by_non_admin() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);
    update_near_price(&mut vcontract, dollars(5));

    set_predecessor(&mut context, Alice);
    vcontract.set_position_limits(
        near_id(),
        AssetPositionLimits {
            long: Limits {
                max: dollars(100),
                min: dollars(0),
            },
            short: Limits {
                max: dollars(42),
                min: dollars(10),
            },
        },
    );
}
