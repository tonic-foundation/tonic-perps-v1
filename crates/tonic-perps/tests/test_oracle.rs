mod common;

use common::*;

#[test]
fn test_oracle_max_change() {
    let (mut context, mut vcontract) = setup();
    set_predecessor(&mut context, Admin);

    vcontract.set_max_asset_price_change(near_id(), Some(U128(2000)));

    vcontract
        .contract_mut()
        .add_liquidity(&AssetId::NEAR, near(100));
    update_near_price(&mut vcontract, dollars(5));

    let assets = vcontract.get_assets();

    let near = assets.iter().find(|asset| asset.id == "near");

    assert_eq!(dollars(5), near.unwrap().average_price.0);

    update_near_price(&mut vcontract, dollars(10));

    let assets = vcontract.get_assets();

    let near = assets.iter().find(|asset| asset.id == "near");

    assert!(dollars(6) > near.unwrap().average_price.0);
}
