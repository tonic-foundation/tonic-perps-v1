use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::edit_reserved_amount)]
pub struct EditReservedAmount {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    account_id: String,
    amount_native: String,
    new_reserved_amount_native: String,
    increase: bool,
    asset_id: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::EditReservedAmountEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::edit_reserved_amount::table)
        .values(&EditReservedAmount {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            amount_native: ev.amount_native.to_string(),
            new_reserved_amount_native: ev.new_reserved_amount_native.to_string(),
            increase: ev.increase,
            asset_id: ev.asset_id,
        })
        .execute(conn)?;

    Ok(())
}
