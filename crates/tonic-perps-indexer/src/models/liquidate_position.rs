use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::event_types;

use crate::schema;
use crate::schema::*;

use super::util::base58_encode;

#[derive(Insertable)]
#[diesel(table_name = perp_event::liquidate_position_event)]
pub struct LiquidatePositionEvent {
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    liquidator_id: String,
    owner_id: String,
    position_id: String,
    collateral_token: String,
    underlying_token: String,
    is_long: bool,
    size_usd: String,
    collateral_usd: String,
    reserve_amount_delta_native: String,
    liquidation_price_usd: String,
    liquidator_reward_usd: String,
    liquidator_reward_native: String,
    fees_native: String,
    fees_usd: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: event_types::LiquidatePositionEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::liquidate_position_event::table)
        .values(&LiquidatePositionEvent {
            block_timestamp,
            receipt_id,
            liquidator_id: ev.liquidator_id.to_string(),
            owner_id: ev.owner_id.to_string(),
            position_id: base58_encode(&ev.position_id),
            collateral_token: ev.collateral_token.to_string(),
            underlying_token: ev.underlying_token.to_string(),
            is_long: ev.is_long,
            size_usd: ev.size_usd.0.to_string(),
            collateral_usd: ev.collateral_usd.0.to_string(),
            reserve_amount_delta_native: ev.reserve_amount_delta_native.0.to_string(),
            liquidation_price_usd: ev.liquidation_price_usd.0.to_string(),
            liquidator_reward_usd: ev.liquidator_reward_usd.0.to_string(),
            liquidator_reward_native: ev.liquidator_reward_native.0.to_string(),
            fees_native: ev.fees_native.0.to_string(),
            fees_usd: ev.fees_usd.0.to_string(),
        })
        .execute(conn)?;

    Ok(())
}
