#![allow(dead_code)]
pub use near_sdk::json_types::U128;
pub use near_sdk::test_utils::accounts;
use near_sdk::test_utils::VMContextBuilder;
pub use near_sdk::testing_env;
use near_sdk::{AccountId, Balance};

pub use tonic_perps::*;

/// Convenience enum. Makes it easier to get test account IDs.
///
/// ```ignore
/// let (mut ctx, mut vcontract) = setup_contract();
/// set_predecessor(&mut ctx, Alice);
/// set_deposit(&mut ctx, TENTH_NEAR);
///
/// vcontract.storage_deposit(None, None);
/// assert_eq!(vcontract.get_users_count(), 1, "failed to register");
/// ```
#[repr(usize)]
pub enum TestAccount {
    Alice = 0,
    Bob,
    Admin,
}
pub use TestAccount::*;

pub const NEAR_DENOMINATION: u128 = 10u128.pow(24);
pub const TENTH_NEAR: u128 = NEAR_DENOMINATION / 10;

pub fn get_account(id: TestAccount) -> AccountId {
    accounts(id as usize)
}

pub fn near_id() -> String {
    "near".to_string()
}

pub fn usdc_id() -> String {
    "usdc".to_string()
}

pub fn dollars(amount: u64) -> DollarBalance {
    amount as u128 * DOLLAR_DENOMINATION
}

pub fn near(amount: u64) -> Balance {
    amount as u128 * NEAR_DENOMINATION
}

pub fn lp_tokens(amount: f64) -> Balance {
    (amount * (LP_TOKEN_DENOMINATION as f64)) as u128
}

// Allows for modifying the environment of the mocked blockchain
pub fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id);
    builder
}

pub fn set_predecessor(context: &mut VMContextBuilder, account: TestAccount) {
    testing_env!(context.predecessor_account_id(get_account(account)).build());
}

pub fn set_signer(context: &mut VMContextBuilder, account: TestAccount) {
    testing_env!(context.signer_account_id(get_account(account)).build());
}

pub fn set_predecessor_token(context: &mut VMContextBuilder, account: String) {
    testing_env!(context
        .predecessor_account_id(AccountId::new_unchecked(account))
        .build());
}

pub fn set_deposit(context: &mut VMContextBuilder, amount: Balance) {
    testing_env!(context.attached_deposit(amount).build());
}

pub fn update_near_price(vcontract: &mut VContract, price: DollarBalance) {
    update_asset_price(vcontract, near_id(), price);
}

pub fn update_asset_price(vcontract: &mut VContract, asset_id: String, price: DollarBalance) {
    vcontract.update_index_price(vec![UpdateIndexPriceRequest {
        asset_id,
        price: U128::from(price),
        spread: None,
    }]);
}

/// Return a handle to the mocked blockchain context + a contract owned by
/// [TestAccount::Owner].
pub fn setup() -> (VMContextBuilder, VContract) {
    let mut context = get_context(get_account(Admin));
    // Initialize the mocked blockchain
    testing_env!(context.build());

    // One day
    context.block_timestamp(std::time::Duration::from_secs(60 * 60 * 24).as_nanos() as u64);

    set_predecessor(&mut context, Admin);

    let mut vcontract = VContract::new(get_account(Admin));
    vcontract.set_fee_parameters(FeeParameters {
        tax_bps: 0,
        stable_tax_bps: 0,
        mint_burn_fee_bps: 0,
        swap_fee_bps: 0,
        stable_swap_fee_bps: 0,
        margin_fee_bps: 0,
    });
    vcontract.add_asset(near_id(), 24, false, 50);
    vcontract.add_asset(usdc_id(), 6, true, 50);
    vcontract.add_price_oracle(get_account(Admin));
    vcontract.add_admin(get_account(Admin), AdminRole::FullAdmin);
    vcontract.set_default_stablecoin("usdc".to_string());
    vcontract.set_state(ContractState::Running);
    vcontract.set_private_liquidation_only(false);

    vcontract.update_index_price(vec![UpdateIndexPriceRequest {
        asset_id: usdc_id(),
        price: U128::from(dollars(1)), // 1 USD
        spread: None,
    }]);

    vcontract.set_shortable(near_id(), true);

    (context, vcontract)
}
