use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::edit_pool_balance)]
pub struct EditPoolBalance {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    amount_native: String,
    new_pool_balance_native: String,
    increase: bool,
    asset_id: String,
    account_id: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::EditPoolBalanceEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::edit_pool_balance::table)
        .values(&EditPoolBalance {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            amount_native: ev.amount_native.to_string(),
            new_pool_balance_native: ev.new_pool_balance_native.to_string(),
            increase: ev.increase,
            asset_id: ev.asset_id,
        })
        .execute(conn)?;

    Ok(())
}
