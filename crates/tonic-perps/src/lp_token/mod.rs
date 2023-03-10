mod ft;
mod mint;
mod storage;

use crate::{borsh, env, AccountId, Balance, BorshDeserialize, BorshSerialize};
pub use ft::*;
pub use mint::*;
use near_contract_standards::fungible_token::events::FtTransfer;
use near_sdk::{collections::LookupMap, json_types::U128, require, IntoStorageKey};

/// Implementation of a FungibleToken standard.
/// https://docs.rs/near-contract-standards/latest/src/near_contract_standards/fungible_token/core_impl.rs.html
///
/// Allows to include NEP-141 compatible token to any contract.
/// There are next traits that any contract may implement:
///     - FungibleTokenCore -- interface with ft_transfer methods. FungibleToken provides methods for it.
///     - FungibleTokenMetaData -- return metadata for the token in NEP-148, up to contract to implement.
///
/// For example usage, see examples/fungible-token/src/lib.rs.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleTokenFreeStorage {
    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, Balance>,

    /// Total supply of the all token.
    pub total_supply: Balance,
}

impl FungibleTokenFreeStorage {
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            accounts: LookupMap::new(prefix),
            total_supply: 0,
        }
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        self.accounts.get(account_id).unwrap_or(0)
    }

    pub fn internal_save_balance(&mut self, account_id: &AccountId, balance: Balance) {
        if balance > 0 {
            self.accounts.insert(account_id, &balance);
        } else {
            self.accounts.remove(account_id);
        }
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            self.internal_save_balance(account_id, new_balance);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("Balance overflow");
        }
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.internal_save_balance(account_id, new_balance);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("The account doesn't have enough balance");
        }
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(
            sender_id != receiver_id,
            "Sender and receiver should be different"
        );
        require!(amount > 0, "The amount should be a positive number");
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        FtTransfer {
            old_owner_id: sender_id,
            new_owner_id: receiver_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }
}
