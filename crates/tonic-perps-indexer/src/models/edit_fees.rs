use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;
use tonic_perps_sdk::prelude::FeeType;

use crate::schema;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::edit_fees)]
pub struct EditFees {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    account_id: String,
    fee_native: String,
    fee_usd: String,
    fee_type: String,
    new_accumulated_fees_native: String,
    new_accumulated_fees_usd: String,
    increase: bool,
    asset_id: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::EditFeesEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::edit_fees::table)
        .values(&EditFees {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            fee_native: ev.fee_native.to_string(),
            fee_usd: ev.fee_usd.to_string(),
            fee_type: match ev.fee_type {
                FeeType::Burn => "burn".to_string(),
                FeeType::Funding => "funding".to_string(),
                FeeType::Mint => "mint".to_string(),
                FeeType::Position => "position".to_string(),
                FeeType::Swap => "swap".to_string(),
                FeeType::WithrawFee => "withdraw_fee".to_string(),
            },
            new_accumulated_fees_native: ev.new_accumulated_fees_native.to_string(),
            new_accumulated_fees_usd: ev.new_accumulated_fees_usd.to_string(),
            increase: ev.increase,
            asset_id: ev.asset_id,
        })
        .execute(conn)?;

    Ok(())
}
