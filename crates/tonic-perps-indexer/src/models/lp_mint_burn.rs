use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::QueryResult;

use tonic_perps_sdk::prelude::{MintBurnDirection as DexMintBurnDirection, MintBurnLpEvent};

use crate::schema;
use crate::schema::*;

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[DieselTypePath = "crate::schema::perp_event::sql_types::MintBurnDirection"]
pub enum DbMintBurnDirection {
    // these exact names matter
    Mint,
    Burn,
}

impl From<DexMintBurnDirection> for DbMintBurnDirection {
    fn from(d: DexMintBurnDirection) -> Self {
        match d {
            DexMintBurnDirection::Burn => DbMintBurnDirection::Burn,
            DexMintBurnDirection::Mint => DbMintBurnDirection::Mint,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = perp_event::lp_mint_burn_event)]
struct LpMintBurnEvent {
    receipt_id: String,
    account_id: String,
    block_timestamp: NaiveDateTime,
    direction: DbMintBurnDirection,
    amount_in: String,
    amount_out: String,
    /// Fee amount in the fee token (token_in if mint, token_out if burn)
    fees: String,
    fees_usd: String,
    fees_bps: i32,
    lp_price_usd: String,
    token_in: String,
    token_out: String,
}

pub fn save(
    conn: &mut PgConnection,
    receipt_id: String,
    block_timestamp: NaiveDateTime,
    ev: MintBurnLpEvent,
) -> QueryResult<()> {
    diesel::insert_into(schema::perp_event::lp_mint_burn_event::table)
        .values(&LpMintBurnEvent {
            receipt_id,
            block_timestamp,
            account_id: ev.account_id.to_string(),
            amount_in: ev.amount_in.0.to_string(),
            token_in: ev.token_in,
            amount_out: ev.amount_out.0.to_string(),
            token_out: ev.token_out,
            direction: ev.direction.into(),
            fees: ev.fees.0.to_string(),
            fees_usd: ev.fees_usd.0.to_string(),
            fees_bps: ev.fees_bps as i32, // not possible to overflow
            lp_price_usd: ev.lp_price_usd.0.to_string(),
        })
        .execute(conn)?;

    Ok(())
}
