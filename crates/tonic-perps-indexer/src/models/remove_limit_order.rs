use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

use super::util::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::remove_limit_order_event)]
pub struct RemoveLimitOrderEvent {
    account_id: String,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    underlying_token: String,
    limit_order_id: String,
    reason: String,
    liquidator_id: Option<String>,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::RemoveLimitOrderEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::remove_limit_order_event::table)
        .values(&RemoveLimitOrderEvent {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            underlying_token: ev.underlying_token,
            limit_order_id: base58_encode(&ev.limit_order_id),
            reason: ev.reason.to_string(),
            liquidator_id: ev.liquidator_id.map(|e| e.to_string()),
        })
        .execute(conn)?;

    Ok(())
}
