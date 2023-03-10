mod common;

use common::*;

use near_sdk::env;
#[test]
fn test_require_predecessor() {
    let (mut context, mut _vcontract) = setup();
    set_predecessor(&mut context, Admin);
    require_predecessor!(get_account(Admin));
}

#[test]
#[should_panic(expected = "admin plz")]
fn test_require_predecessor_panic() {
    let (mut context, mut _vcontract) = setup();
    set_predecessor(&mut context, Alice);
    require_predecessor!(get_account(Admin), "admin plz");
}
