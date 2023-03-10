use std::collections::VecDeque;
use std::time::Duration;

use serde::Serialize;

use crate::{borsh, env, BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Copy)]
pub enum TransferType {
    Deposit,
    Withdraw,
}

/// Represents a withdrawal or a deposit
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
pub struct TokenTransfer {
    /// Amount of tokens
    amount: u128,

    /// If true, this is a deposit, else it's a withdrawal
    transfer_type: TransferType,

    /// In ms
    timestamp: u64,
}

impl TokenTransfer {
    pub fn new(amount: u128, timestamp: u64, transfer_type: TransferType) -> Self {
        TokenTransfer {
            amount,
            timestamp,
            transfer_type,
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn amount(&self) -> u128 {
        self.amount
    }

    pub fn transfer_type(&self) -> TransferType {
        self.transfer_type
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
pub struct TokenTransferHistory {
    /// Double-ended queue
    transfers: VecDeque<TokenTransfer>,

    /// Sliding window duration in ms
    sliding_window_duration_ms: u64,
}

impl TokenTransferHistory {
    pub fn new(sliding_window_duration: u64) -> Self {
        TokenTransferHistory {
            transfers: VecDeque::new(),
            sliding_window_duration_ms: sliding_window_duration,
        }
    }

    /// Get the total amount withdrawn in the last sliding window.
    /// If more money has been deposited than withdrawn, the result will be zero.
    pub fn amount(&self) -> u128 {
        // We check time in case clean was not called recently
        // We don't clean because we do not have a mutable ref
        let time_limit = self.sliding_window_limit(env::block_timestamp_ms());

        let withdrawals = self
            .transfers
            .iter()
            .filter_map(|e| {
                if e.timestamp > time_limit && matches!(e.transfer_type(), TransferType::Withdraw) {
                    Some(e.amount())
                } else {
                    None
                }
            })
            .sum::<u128>();
        let deposits = self
            .transfers
            .iter()
            .filter_map(|e| {
                if e.timestamp > time_limit && matches!(e.transfer_type(), TransferType::Deposit) {
                    Some(e.amount())
                } else {
                    None
                }
            })
            .sum::<u128>();
        withdrawals.saturating_sub(deposits)
    }

    /// Register a new withdrawal
    pub fn push(&mut self, withdrawal: TokenTransfer) {
        self.transfers.push_back(withdrawal);
    }

    /// Remove all withdrawals made outside of the sliding window
    pub fn clean(&mut self, current_timestamp_ms: u64) {
        let time_limit = self.sliding_window_limit(current_timestamp_ms);

        while self.transfers.get(0).is_some()
            && self.transfers.get(0).unwrap().timestamp() < time_limit
        {
            self.transfers.pop_front();
        }
    }

    fn sliding_window_limit(&self, current_timestamp_ms: u64) -> u64 {
        current_timestamp_ms.saturating_sub(self.sliding_window_duration_ms)
    }

    pub fn update_sliding_window_duration(&mut self, sliding_window_duration: u64) {
        self.sliding_window_duration_ms = sliding_window_duration;
    }
}

impl Default for TokenTransferHistory {
    fn default() -> Self {
        TokenTransferHistory::new(Duration::from_secs(60 * 60).as_millis() as u64)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_withdrawal_history_clean() {
        let mut withdrawal_history = TokenTransferHistory::default();
        withdrawal_history.push(TokenTransfer::new(
            100,
            Duration::from_secs(1000).as_millis() as u64,
            TransferType::Withdraw,
        ));
        withdrawal_history.push(TokenTransfer::new(
            100,
            Duration::from_secs(3000).as_millis() as u64,
            TransferType::Withdraw,
        ));
        assert_eq!(withdrawal_history.transfers.len(), 2);
        assert_eq!(withdrawal_history.amount(), 200);

        withdrawal_history.push(TokenTransfer::new(
            100,
            Duration::from_secs(5000).as_millis() as u64,
            TransferType::Deposit,
        ));
        withdrawal_history.push(TokenTransfer::new(
            200,
            Duration::from_secs(7000).as_millis() as u64,
            TransferType::Withdraw,
        ));
        assert_eq!(withdrawal_history.transfers.len(), 4);
        assert_eq!(withdrawal_history.amount(), 300);

        // Sliding window 3600s, time limit - 4400s, remove 2 withdrawals.
        withdrawal_history.clean(Duration::from_secs(8000).as_millis() as u64);
        assert_eq!(withdrawal_history.transfers.len(), 2);
        assert_eq!(withdrawal_history.amount(), 100);
    }
}
