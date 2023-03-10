use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::prelude::{
    EditPositionDirection as DexEditPositionDirection, EditPositionEvent as DexEditPositionEvent,
    EditPositionState as DexEditPositionState,
};

use crate::schema;
use crate::schema::*;

use super::util::*;

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[DieselTypePath = "crate::schema::perp_event::sql_types::EditPositionDirection"]
pub enum DbEditPositionDirection {
    // these exact names matter
    Increase,
    Decrease,
}

impl From<DexEditPositionDirection> for DbEditPositionDirection {
    fn from(d: DexEditPositionDirection) -> Self {
        match d {
            DexEditPositionDirection::Decrease => DbEditPositionDirection::Decrease,
            DexEditPositionDirection::Increase => DbEditPositionDirection::Increase,
        }
    }
}

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[DieselTypePath = "crate::schema::perp_event::sql_types::EditPositionState"]
pub enum DbEditPositionState {
    // these exact names matter
    Created,
    Closed,
    Open,
}

impl From<DexEditPositionState> for DbEditPositionState {
    fn from(s: DexEditPositionState) -> Self {
        match s {
            DexEditPositionState::Closed => DbEditPositionState::Closed,
            DexEditPositionState::Created => DbEditPositionState::Created,
            DexEditPositionState::Open => DbEditPositionState::Open,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = perp_event::edit_position_event)]
struct EditPositionEvent {
    receipt_id: String,
    account_id: String,
    block_timestamp: NaiveDateTime,
    position_id: String,
    direction: DbEditPositionDirection,
    /// State as a result of editing
    state: DbEditPositionState,
    collateral_token: String,
    underlying_token: String,
    collateral_delta_native: String,
    collateral_delta_usd: String,
    size_delta_usd: String,
    new_size_usd: String,
    is_long: bool,
    price_usd: String,
    total_fee_usd: String,
    margin_fee_usd: String,
    position_fee_usd: String,

    usd_out: String,
    realized_pnl_to_date_usd: String,
    adjusted_delta_usd: String,

    /// Total fee in the fee token.
    total_fee_native: String,

    /// Margin fee in the fee token.
    margin_fee_native: String,

    /// Position fee in the fee token.
    position_fee_native: String,
    referral_code: Option<String>,
    limit_order_id: Option<String>,
    liquidator_id: Option<String>,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: DexEditPositionEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::edit_position_event::table)
        .values(&EditPositionEvent {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            direction: ev.direction.into(),
            position_id: base58_encode(&ev.position_id),
            state: ev.state.into(),
            collateral_token: ev.collateral_token,
            underlying_token: ev.underlying_token,
            collateral_delta_native: ev.collateral_delta_native.0.to_string(),
            collateral_delta_usd: ev.collateral_delta_usd.0.to_string(),
            size_delta_usd: ev.size_delta_usd.0.to_string(),
            new_size_usd: ev.new_size_usd.0.to_string(),
            is_long: ev.is_long,
            price_usd: ev.price_usd.0.to_string(),
            total_fee_usd: ev.total_fee_usd.0.to_string(),
            margin_fee_usd: ev.margin_fee_usd.0.to_string(),
            position_fee_usd: ev.position_fee_usd.0.to_string(),
            total_fee_native: ev.total_fee_native.0.to_string(),
            margin_fee_native: ev.margin_fee_native.0.to_string(),
            position_fee_native: ev.position_fee_native.0.to_string(),
            referral_code: ev.referral_code,
            usd_out: ev.usd_out.0.to_string(),
            realized_pnl_to_date_usd: ev.realized_pnl_to_date_usd.0.to_string(),
            adjusted_delta_usd: ev.adjusted_delta_usd.0.to_string(),
            limit_order_id: ev.limit_order_id.map(|id| id.to_string()),
            liquidator_id: ev.liquidator_id.map(|id| id.to_string()),
        })
        .execute(conn)?;

    Ok(())
}
