use diesel::{Connection, PgConnection};
use rand::Rng;
use std::convert::TryInto;

use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::{
    ExecutionStatusView, ReceiptEnumView, ReceiptView,
};
use near_lake_framework::near_indexer_primitives::IndexerExecutionOutcomeWithReceipt;
use tracing::{error, info};

use tonic_perps_sdk::event_types::EventType as PerpsEventType;

use crate::constants::TARGET;
use crate::db::PgPool;
use crate::event;
use crate::models;

pub struct Worker {
    contract_id: String,
    db_pool: PgPool,
}

impl Worker {
    pub fn new(contract_id: String, db_pool: PgPool) -> Self {
        Self {
            contract_id,
            db_pool,
        }
    }
}

impl Worker {
    pub async fn process_message(
        &self,
        streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
    ) {
        let mut conn = self.db_pool.get().expect("Unable to get connection");
        let block_height = streamer_message.block.header.height;

        // process each block as a transaction
        let res = conn.transaction::<_, diesel::result::Error, _>(|tx_conn| {
            for shard in streamer_message.shards {
                for o in shard.receipt_execution_outcomes {
                    if self.should_save(&o) {
                        let block_timestamp_ns = streamer_message.block.header.timestamp_nanosec;
                        save_execution_outcome(tx_conn, o, block_timestamp_ns)?;
                    }
                }
            }
            // unique constraint on block number will roll back (ie, skip) dupes
            models::latest_processed_block::save(tx_conn, block_height)?;

            Ok(())
        });

        if let Err(e) = res {
            error!(
                target: TARGET,
                "Error saving block {}, {:?}", block_height, e
            );
        }
    }

    pub fn should_save(&self, o: &IndexerExecutionOutcomeWithReceipt) -> bool {
        if matches!(
            o.execution_outcome.outcome.status,
            ExecutionStatusView::Unknown | ExecutionStatusView::Failure(_)
        ) {
            return false;
        }
        if o.execution_outcome.outcome.logs.is_empty() {
            return false;
        }
        o.receipt.receiver_id.to_string() == self.contract_id
    }
}

fn save_execution_outcome(
    conn: &mut PgConnection,
    o: IndexerExecutionOutcomeWithReceipt,
    block_timestamp_ns: u64,
) -> diesel::result::QueryResult<()> {
    for log in o.execution_outcome.outcome.logs {
        if !event::is_event_log(&log) {
            continue;
        }
        if let Ok(ev) = event::parse_event(&log) {
            save_event(conn, ev, &o.receipt, block_timestamp_ns)?
        }
    }

    Ok(())
}

fn save_event(
    conn: &mut PgConnection,
    ev: PerpsEventType,
    receipt: &ReceiptView,
    block_timestamp_ns: u64,
) -> diesel::result::QueryResult<()> {
    use models::*;

    if !matches!(receipt.receipt, ReceiptEnumView::Action { .. }) {
        return Ok(());
    }

    let receipt_id = receipt.receipt_id.to_string();
    let (timestamp_ms, excess_ns) = (
        block_timestamp_ns / 1_000_000_000,
        block_timestamp_ns % 1_000_000_000,
    );
    let timestamp = NaiveDateTime::from_timestamp_opt(
        timestamp_ms.try_into().unwrap(),
        excess_ns.try_into().unwrap(),
    )
    .unwrap();

    let now = std::time::Instant::now();
    match ev {
        PerpsEventType::LpPriceUpdate(ev) => {
            lp_price_update::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::MintBurnLp(ev) => lp_mint_burn::save(conn, receipt_id, timestamp, ev)?,
        PerpsEventType::EditPosition(ev) => edit_position::save(conn, receipt_id, timestamp, ev)?,
        PerpsEventType::CreateReferralCode(ev) => {
            create_referral_code::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::PlaceLimitOrder(ev) => {
            place_limit_order::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::RemoveLimitOrder(ev) => {
            remove_limit_order::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::Swap(ev) => swap::save(conn, receipt_id, timestamp, ev)?,
        PerpsEventType::LiquidatePosition(ev) => {
            liquidate_position::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::EditPoolBalance(ev) => {
            edit_pool_balance::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::EditReservedAmount(ev) => {
            edit_reserved_amount::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::EditGuaranteedUsd(ev) => {
            edit_guaranteed_usd::save(conn, receipt_id, timestamp, ev)?
        }
        PerpsEventType::EditFees(ev) => edit_fees::save(conn, receipt_id, timestamp, ev)?,
        PerpsEventType::TokenDepositWithdraw(ev) => {
            token_deposit_withdraw::save(conn, receipt_id, timestamp, ev)?
        }
        _ => info!(target: TARGET, "unsupported event {:?}", ev),
    };
    // log 1% of events
    let mut rng = rand::thread_rng();
    if rng.gen_range(0.0..1.0) < 0.01 {
        let elapsed = now.elapsed();
        info!(target: TARGET, "Wrote event in {:.2?}", elapsed);
    }

    Ok(())
}
