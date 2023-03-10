use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::edit_guaranteed_usd)]
pub struct EditGuaranteedUsd {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    account_id: String,
    amount_usd: String,
    new_guaranteed_usd: String,
    increase: bool,
    asset_id: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::EditGuaranteedUsdEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::edit_guaranteed_usd::table)
        .values(&EditGuaranteedUsd {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            amount_usd: ev.amount_usd.to_string(),
            new_guaranteed_usd: ev.new_guaranteed_usd.to_string(),
            increase: ev.increase,
            asset_id: ev.asset_id,
        })
        .execute(conn)?;

    Ok(())
}
