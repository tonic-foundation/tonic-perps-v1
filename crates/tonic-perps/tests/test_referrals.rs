mod common;

use common::*;

#[test]
fn set_referral_code() {
    let (mut context, mut vcontract) = setup();
    set_deposit(&mut context, near(1));

    vcontract.set_user_referral_code("test".to_string());
}

#[test]
#[should_panic(expected = "Must provide 0.01 NEAR to set referral code")]
fn set_referral_code_no_deposit() {
    let (_, mut vcontract) = setup();
    vcontract.set_user_referral_code("test".to_string());
}
