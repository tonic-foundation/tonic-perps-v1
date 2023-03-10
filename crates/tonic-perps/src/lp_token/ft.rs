use crate::{near_bindgen, FungibleTokenFreeStorage, VContract, VContractExt};
use near_contract_standards::fungible_token::events::FtTransfer;
use near_contract_standards::fungible_token::receiver::ext_ft_receiver;
use near_contract_standards::fungible_token::resolver::{ext_ft_resolver, FungibleTokenResolver};
/// Implement fungible token standard for the LP token
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
};
use near_sdk::{assert_one_yocto, env, require, AccountId, Balance, PromiseResult};
use near_sdk::{json_types::U128, Gas, PromiseOrValue};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
const LP_TOKEN_DECIMALS: u8 = 18;
pub const LP_TOKEN_DENOMINATION: u128 = 10u128.pow(LP_TOKEN_DECIMALS as u32);
const LP_TOKEN_NAME: &str = "Tonic Index LP Token";
const LP_TOKEN_SYMBOL: &str = "GIN";
/// Min transfer amount if receiver doesn't exists. 0.001GIN
const MIN_TRANSFER_AMOUNT: u128 = 1_000_000_000_000_000;

impl FungibleTokenCore for FungibleTokenFreeStorage {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.assert_min_transfer_amount(&receiver_id, amount);
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        self.assert_min_transfer_amount(&receiver_id, amount);

        require!(
            env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL,
            "More gas is required"
        );
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
        // Initiating receiver's call and the callback
        ext_ft_receiver::ext(receiver_id.clone())
            .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL)
            .ft_on_transfer(sender_id.clone(), amount.into(), msg)
            .then(
                ext_ft_resolver::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .ft_resolve_transfer(sender_id, receiver_id, amount.into()),
            )
            .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.accounts.get(&account_id).unwrap_or(0).into()
    }
}

impl FungibleTokenFreeStorage {
    /// Internal method that returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    /// As current implementation of fungible token doesn't require user's registration,
    /// there is no case of burning tokens.
    pub fn internal_ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> u128 {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                self.internal_save_balance(&receiver_id, receiver_balance - refund_amount);

                let sender_balance = self.internal_unwrap_balance_of(sender_id);
                self.internal_save_balance(sender_id, sender_balance + refund_amount);

                FtTransfer {
                    old_owner_id: &receiver_id,
                    new_owner_id: sender_id,
                    amount: &U128(refund_amount),
                    memo: Some("refund"),
                }
                .emit();
                return amount - refund_amount;
            }
        }
        amount
    }

    fn assert_min_transfer_amount(&self, receiver_id: &AccountId, amount: U128) {
        if self.accounts.get(receiver_id).is_none() {
            assert!(
                amount.0 >= MIN_TRANSFER_AMOUNT,
                "Requires min 0.001GIN to transfer as receiver is not registered"
            );
        }
    }
}

// copy-paste of impl_fungible_token_core, which doesn't work with versioned
// contracts
// <COPYPASTE>
#[near_bindgen]
impl FungibleTokenCore for VContract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.contract_mut()
            .lp_token
            .ft_transfer(receiver_id, amount, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.contract_mut()
            .lp_token
            .ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.contract().lp_token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.contract().lp_token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for VContract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let used_amount = self.contract_mut().lp_token.internal_ft_resolve_transfer(
            &sender_id,
            receiver_id,
            amount,
        );
        used_amount.into()
    }
}
// </COPYPASTE>

#[near_bindgen]
impl FungibleTokenMetadataProvider for VContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            reference: None,
            reference_hash: None,
            decimals: LP_TOKEN_DECIMALS,
            name: LP_TOKEN_NAME.into(),
            symbol: LP_TOKEN_SYMBOL.into(),
            icon: None,
        }
    }
}
