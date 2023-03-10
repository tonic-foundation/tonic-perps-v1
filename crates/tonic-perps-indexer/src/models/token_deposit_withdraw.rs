use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::token_deposit_withdraw)]
pub struct TokenDepositWithdraw {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    amount_native: String,
    deposit: bool,
    method: String,
    receiver_id: String,
    account_id: String,
    asset_id: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::TokenDepositWithdrawEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::token_deposit_withdraw::table)
        .values(&TokenDepositWithdraw {
            receipt_id,
            block_timestamp,
            amount_native: ev.amount_native.0.to_string(),
            deposit: ev.deposit,
            method: ev.method,
            receiver_id: ev.receiver_id.to_string(),
            account_id: ev.account_id.to_string(),
            asset_id: ev.asset_id,
        })
        .execute(conn)?;

    Ok(())
}
