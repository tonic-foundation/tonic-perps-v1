use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::lp_price_update_event)]
pub struct LpPriceUpdateEvent {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    price: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::LpPriceUpdateEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::lp_price_update_event::table)
        .values(&LpPriceUpdateEvent {
            receipt_id,
            block_timestamp,
            price: ev.price.0.to_string(),
        })
        .execute(conn)?;

    Ok(())
}
