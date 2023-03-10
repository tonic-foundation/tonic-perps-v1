use crate::{near_bindgen, AccountId, VContract, VContractExt};

use near_contract_standards::storage_management::StorageBalance;
use near_sdk::{env, Promise};

/// 0.125 NEAR
const DEFAULT_STORAGE_BALANCE: u128 = 125_000_000_000_000_000_000_000; // 125m

#[near_bindgen]
impl VContract {
    /// Always returns 125 milliNEAR indicating that user doesn't need to be registered.
    /// It's a workaround for integrations required NEP-125 storage compatibility.
    #[allow(unused_variables)]
    pub fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        Some(StorageBalance {
            total: DEFAULT_STORAGE_BALANCE.into(),
            available: 0.into(),
        })
    }

    /// Mock API method as current token implementation excludes storage tracking,
    /// refund NEAR if it was attached.
    #[allow(unused_variables)]
    #[payable]
    pub fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        if amount > 0 {
            Promise::new(env::predecessor_account_id()).transfer(amount);
        };
        StorageBalance {
            total: DEFAULT_STORAGE_BALANCE.into(),
            available: 0.into(),
        }
    }
}
