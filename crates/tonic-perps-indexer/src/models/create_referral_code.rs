use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::create_referral_code_event)]
pub struct CreateReferralCodeEvent {
    receipt_id: String,
    /// Referral code owner
    account_id: String,
    block_timestamp: NaiveDateTime,
    code: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::CreateReferralCodeEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::create_referral_code_event::table)
        .values(&CreateReferralCodeEvent {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            code: ev.referral_code,
        })
        .execute(conn)?;

    Ok(())
}
