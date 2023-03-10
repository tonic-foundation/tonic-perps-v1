use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

use super::util::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::place_limit_order_event)]
pub struct PlaceLimitOrderEvent {
    account_id: String,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    limit_order_id: String,
    collateral_token: String,
    underlying_token: String,
    order_type: String,
    threshold_type: String,
    collateral_delta_usd: String,
    attached_collateral_native: String,
    size_delta_usd: String,
    price_usd: String,
    expiry: NaiveDateTime,
    is_long: bool,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::PlaceLimitOrderEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::place_limit_order_event::table)
        .values(&PlaceLimitOrderEvent {
            block_timestamp,
            receipt_id,
            account_id: ev.account_id.to_string(),
            limit_order_id: base58_encode(&ev.limit_order_id),
            collateral_token: ev.collateral_token,
            underlying_token: ev.underlying_token,
            order_type: ev.order_type,
            threshold_type: ev.threshold_type,
            collateral_delta_usd: ev.collateral_delta_usd.0.to_string(),
            attached_collateral_native: ev.attached_collateral_native.0.to_string(),
            size_delta_usd: ev.size_delta_usd.0.to_string(),
            price_usd: ev.price_usd.0.to_string(),
            expiry: chrono::NaiveDateTime::from_timestamp_millis(ev.expiry.0 as i64).unwrap(),
            is_long: ev.is_long,
        })
        .execute(conn)?;

    Ok(())
}
