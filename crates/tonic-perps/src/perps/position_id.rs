use crate::{AccountId, AssetId, Base58VecU8};

use std::convert::TryFrom;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use tonic_perps_sdk::impl_base58_serde;

/// Market IDs are sha256 hashes (ie 32 byte arrays)
#[derive(PartialEq, Eq, Hash, PartialOrd, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct PositionId(pub [u8; 32]);

impl PositionId {
    pub fn new(
        account_id: &AccountId,
        collateral_id: &AssetId,
        underlying_id: &AssetId,
        is_long: bool,
        seq: u64,
    ) -> Self {
        let base = format!(
            "{} {} {} {} {}",
            account_id,
            collateral_id.into_string(),
            underlying_id.into_string(),
            if is_long { "long" } else { "short" },
            seq
        );
        PositionId(near_sdk::env::sha256_array(base.as_bytes()))
    }

    pub fn new_unchecked(data: &[u8]) -> Self {
        let mut buf: [u8; 32] = Default::default();
        buf.copy_from_slice(&data[..32]);
        Self(buf)
    }
}

impl_base58_serde!(PositionId);

impl TryFrom<&Vec<u8>> for PositionId {
    type Error = ();

    fn try_from(d: &Vec<u8>) -> Result<Self, Self::Error> {
        if d.len() != 32 {
            Err(())
        } else {
            Ok(PositionId::new_unchecked(d))
        }
    }
}

impl Display for PositionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", near_sdk::bs58::encode(&self.0).into_string())
    }
}

impl Debug for PositionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PositionId<{}>",
            near_sdk::bs58::encode(&self.0).into_string()
        )
    }
}

impl From<Base58VecU8> for PositionId {
    fn from(b: Base58VecU8) -> Self {
        PositionId::try_from(&b.0).expect("malformed position ID")
    }
}

impl From<&Base58VecU8> for PositionId {
    fn from(b: &Base58VecU8) -> Self {
        PositionId::try_from(&b.0).expect("malformed position ID")
    }
}

impl From<PositionId> for Base58VecU8 {
    fn from(m: PositionId) -> Self {
        m.0.to_vec().into()
    }
}

impl From<&PositionId> for Base58VecU8 {
    fn from(m: &PositionId) -> Self {
        m.0.to_vec().into()
    }
}

impl Deref for PositionId {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
