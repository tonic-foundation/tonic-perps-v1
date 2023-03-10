/// Module for working with balance accounting inside the contract.
use near_contract_standards::fungible_token::core::ext_ft_core;
use serde::{Deserialize, Serialize};
use tonic_perps_sdk::prelude::{emit_event, EventType, TokenDepositWithdrawEvent};

use crate::{env, AccountId, Balance, Contract, DollarBalance};

use near_sdk::{Gas, Promise, PromiseOrValue, ONE_YOCTO};
use std::time::Duration;

mod asset;

pub use asset::*;

pub const TGAS_FOR_FT_TRANSFER: u64 = 20;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferInfo {
    receiver_id: AccountId,
    asset_id: AssetId,
    amount: Balance,
}

impl TransferInfo {
    pub fn new(receiver_id: &AccountId, asset_id: &AssetId, amount: Balance) -> Self {
        Self {
            receiver_id: receiver_id.clone(),
            asset_id: asset_id.clone(),
            amount,
        }
    }

    pub fn receiver_id(&self) -> AccountId {
        self.receiver_id.clone()
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id.clone()
    }

    pub fn amount(&self) -> Balance {
        self.amount
    }
}

impl Contract {
    pub(crate) fn get_total_aum(&self) -> DollarBalance {
        self.assets.get_total_aum()
    }

    pub fn update_cumulative_funding_rate(&self, asset: &mut Asset) -> u128 {
        asset.update_cumulative_funding_rate(
            Duration::from_millis(env::block_timestamp_ms()).as_secs(),
            self.funding_interval_seconds.into(),
        )
    }

    pub fn internal_send(&self, transfer_info: TransferInfo, source: &str) -> PromiseOrValue<()> {
        let TransferInfo {
            receiver_id,
            asset_id,
            amount,
        } = transfer_info.clone();

        if amount > 0 {
            emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
                amount_native: amount.into(),
                deposit: false,
                method: source.to_string(),
                receiver_id: receiver_id.clone(),
                account_id: env::current_account_id(),
                asset_id: asset_id.into_string(),
            }));
            match transfer_info.asset_id {
                AssetId::NEAR => Promise::new(receiver_id.clone()).transfer(amount).into(),
                AssetId::Ft(_) => self.internal_send_ft(transfer_info).into(),
            }
        } else {
            PromiseOrValue::Value(())
        }
    }

    pub fn internal_send_ft(&self, transfer_info: TransferInfo) -> Promise {
        let asset_id = transfer_info.asset_id.into_string().parse().unwrap();
        // Example of the new API
        // https://cs.github.com/octopus-network/octopus-appchain-registry/blob/1a1b19d265ec3dd0f4dff5d058ab2a2c7f646f48/appchain-registry/src/voter_actions.rs#L35
        ext_ft_core::ext(asset_id)
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(Gas::ONE_TERA * TGAS_FOR_FT_TRANSFER)
            .with_unused_gas_weight(0)
            .ft_transfer(transfer_info.receiver_id, transfer_info.amount.into(), None)
    }
}
