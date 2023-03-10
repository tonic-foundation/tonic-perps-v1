use std::fmt::{Display, Formatter, Result};

use tonic_perps_sdk::prelude::TokenDepositWithdrawEvent;

use crate::{
    borsh, emit_event, env, near_bindgen, AccountId, AssetId, Balance, BorshDeserialize,
    BorshSerialize, Contract, CreateReferralCodeEvent, Deserialize, EventType, Serialize,
    SetReferralCodeEvent, SetReferrerTierEvent, TransferInfo, VContract, VContractExt,
};

const MAX_REFERRAL_CODE_LENGTH: u8 = 32;
const CREATE_REFERRER_FEE: Balance = 1_000_000_000_000_000_000_000_000 / 20;
const SET_REFERRER_FEE: Balance = 1_000_000_000_000_000_000_000_000 / 100;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ReferrerTier {
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
}

impl Display for ReferrerTier {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Contract {
    pub fn set_user_referral_code(&mut self, account_id: AccountId, referral_code: String) {
        self.check_referral_code(&referral_code);

        self.user_referral_code.insert(&account_id, &referral_code);

        emit_event(EventType::SetReferralCode(SetReferralCodeEvent {
            account_id,
            referral_code,
        }));
    }

    pub fn create_referral_code(&mut self, account_id: AccountId, referral_code: String) {
        self.check_referral_code(&referral_code);

        if self
            .referral_code_owners
            .insert(&referral_code, &(account_id.clone(), ReferrerTier::Tier1))
            .is_some()
        {
            env::panic_str("Referral code already exists");
        }

        emit_event(EventType::CreateReferralCode(CreateReferralCodeEvent {
            account_id,
            referral_code,
        }));
    }

    pub fn set_referral_tier(&mut self, referral_code: String, tier: ReferrerTier) {
        self.assert_admin();
        if let Some((account_id, _)) = self.referral_code_owners.get(&referral_code) {
            self.referral_code_owners
                .insert(&referral_code, &(account_id.clone(), tier));

            emit_event(EventType::SetReferrerTier(SetReferrerTierEvent {
                account_id,
                referral_code,
                tier: tier.to_string(),
            }))
        }
    }

    fn check_referral_code(&self, referral_code: &String) {
        if referral_code.is_empty() {
            env::panic_str("Referral code length can not be 0");
        }
        if referral_code.len() > MAX_REFERRAL_CODE_LENGTH.into() {
            env::panic_str("Referral code exceeds maximum length");
        }
    }
}

#[near_bindgen]
impl VContract {
    #[payable]
    pub fn set_user_referral_code(&mut self, referral_code: String) {
        let contract = self.contract_mut();
        contract.assert_running();
        assert!(
            env::attached_deposit() >= SET_REFERRER_FEE,
            "Must provide 0.01 NEAR to set referral code"
        );

        contract.set_user_referral_code(env::signer_account_id(), referral_code);
    }

    #[payable]
    pub fn create_referral_code(&mut self, referral_code: String) {
        let contract = self.contract_mut();
        contract.assert_running();
        assert!(
            env::attached_deposit() >= CREATE_REFERRER_FEE,
            "Must provide 0.05 NEAR to create referral code"
        );
        let account_id = env::predecessor_account_id();
        contract.create_referral_code(account_id.clone(), referral_code);

        let refund = env::attached_deposit() - CREATE_REFERRER_FEE;
        let transfer_info = TransferInfo::new(&account_id, &AssetId::NEAR, refund);
        contract.internal_send(transfer_info, "create_referral_code");

        emit_event(EventType::TokenDepositWithdraw(TokenDepositWithdrawEvent {
            amount_native: CREATE_REFERRER_FEE.into(),
            deposit: true,
            method: "create_referral_code".to_string(),
            receiver_id: env::current_account_id(),
            account_id,
            asset_id: AssetId::NEAR.into_string(),
        }));
    }

    pub fn set_referral_tier(&mut self, referral_code: String, tier: ReferrerTier) {
        let contract = self.contract_mut();
        contract.assert_running();
        contract.set_referral_tier(referral_code, tier);
    }

    pub fn get_referral_owner(&self, referral_code: String) -> Option<AccountId> {
        self.contract()
            .referral_code_owners
            .get(&referral_code)
            .map(|(code, _)| code)
    }
}
