use crate::{Base58VecU8, LimitOrder, ThresholdType};

use std::convert::TryInto;
use std::fmt::Display;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use tonic_perps_sdk::impl_base58_serde;

/// Ordered ID
///
/// The first bit stores whether the order is long
/// or short. The second bit describes the threshold type.
/// The next 64 bits store `price` as a
/// 64 bit unsigned integer (overflow problem ???).
/// The last 62 bits store a sequence number.
/// `price` comes before sequence number for easy ordering.
#[derive(
    BorshDeserialize, BorshSerialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Debug,
)]
pub struct LimitOrderId(pub u128);

const SEQUENCE_MASK: u128 = (1u128 << 62) - 1;

// 64 bits starting at the second bit
const PRICE_MASK: u128 = ((1u128 << 126) - 1) - ((1u128 << 62) - 1);

impl LimitOrderId {
    pub fn new(limit_order: &LimitOrder, seq_number: u64) -> LimitOrderId {
        let first_bit = if limit_order.is_long { 0 } else { 1u128 << 127 };
        let second_bit = if matches!(limit_order.threshold, ThresholdType::Below) {
            0
        } else {
            1u128 << 126
        };

        let seq = SEQUENCE_MASK & (seq_number as u128);

        LimitOrderId(first_bit | second_bit | ((limit_order.price << 62) & PRICE_MASK) | seq)
    }

    pub fn new_unchecked(data: u128) -> LimitOrderId {
        Self(data)
    }

    fn first_bit(is_long: bool) -> u128 {
        match is_long {
            true => 0,
            false => 1u128 << 127,
        }
    }

    fn second_bit(threshold: ThresholdType) -> u128 {
        match threshold {
            ThresholdType::Below => 0,
            ThresholdType::Above => 1u128 << 126,
        }
    }

    pub fn get_order_id_parts(&self) -> (bool, ThresholdType, u64, u64) {
        let is_long_part = self.0 >> 127;
        let threshold_part = self.0 >> 126 & 1;
        let price_part = ((self.0 & PRICE_MASK) >> 62) as u64;
        let sequence_part = (SEQUENCE_MASK & self.0) as u64;

        let is_long = is_long_part == 0;
        let threshold = if threshold_part == 0 {
            ThresholdType::Below
        } else {
            ThresholdType::Above
        };

        (is_long, threshold, price_part, sequence_part)
    }

    pub fn get_min_id_from_price(
        price: u128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> LimitOrderId {
        let first_bit: u128 = LimitOrderId::first_bit(is_long);
        let second_bit: u128 = LimitOrderId::second_bit(threshold);
        LimitOrderId::new_unchecked(first_bit | second_bit | (price << 62))
    }

    pub fn get_max_id_from_price(
        price: u128,
        is_long: bool,
        threshold: ThresholdType,
    ) -> LimitOrderId {
        let first_bit: u128 = LimitOrderId::first_bit(is_long);
        let second_bit: u128 = LimitOrderId::second_bit(threshold);
        LimitOrderId::new_unchecked(first_bit | second_bit | (price << 62) | SEQUENCE_MASK)
    }

    pub fn get_min_id(is_long: bool, threshold: ThresholdType) -> LimitOrderId {
        let first_bit: u128 = LimitOrderId::first_bit(is_long);
        let second_bit: u128 = LimitOrderId::second_bit(threshold);
        LimitOrderId::new_unchecked(first_bit | second_bit)
    }

    pub fn get_max_id(is_long: bool, threshold: ThresholdType) -> LimitOrderId {
        let first_bit: u128 = LimitOrderId::first_bit(is_long);
        let second_bit: u128 = LimitOrderId::second_bit(threshold);
        LimitOrderId::new_unchecked(first_bit | second_bit | (u128::MAX >> 2))
    }
}

impl_base58_serde!(LimitOrderId);

impl Display for LimitOrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &near_sdk::bs58::encode(&self.0.to_be_bytes()).into_string()
        )
    }
}

impl From<LimitOrderId> for Base58VecU8 {
    fn from(id: LimitOrderId) -> Self {
        id.0.to_be_bytes().to_vec().into()
    }
}

impl From<&LimitOrderId> for Base58VecU8 {
    fn from(id: &LimitOrderId) -> Self {
        id.0.to_be_bytes().to_vec().into()
    }
}

impl From<Base58VecU8> for LimitOrderId {
    fn from(bytes: Base58VecU8) -> Self {
        LimitOrderId(u128::from_be_bytes(bytes.0.try_into().unwrap()))
    }
}
