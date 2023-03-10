use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::swap_event)]
pub struct SwapEvent {
    account_id: String,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    token_in: String,
    token_out: String,
    amount_in_native: String,
    amount_out_native: String,
    amount_in_usd: String,
    amount_out_usd: String,
    fees_native: String,
    fees_usd: String,
    fee_bps: i32,
    referral_code: Option<String>,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::SwapEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::swap_event::table)
        .values(&SwapEvent {
            block_timestamp,
            receipt_id,
            account_id: ev.account_id.to_string(),
            token_in: ev.token_in,
            token_out: ev.token_out,
            amount_in_native: ev.amount_in_native.0.to_string(),
            amount_out_native: ev.amount_out_native.0.to_string(),
            amount_in_usd: ev.amount_in_usd.0.to_string(),
            amount_out_usd: ev.amount_out_usd.0.to_string(),
            fees_native: ev.fees_native.0.to_string(),
            fees_usd: ev.fees_usd.0.to_string(),
            fee_bps: ev.fee_bps as i32,
            referral_code: ev.referral_code,
        })
        .execute(conn)?;

    Ok(())
}
