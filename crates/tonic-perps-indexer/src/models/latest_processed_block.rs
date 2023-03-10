use diesel::prelude::*;
use diesel::result::QueryResult;

use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = perp_event::indexer_processed_block)]
pub struct IndexerProcessedBlock {
    block_height: i32,
}

pub fn get(conn: &mut PgConnection) -> QueryResult<u64> {
    use perp_event::indexer_processed_block::dsl::*;
    let res = indexer_processed_block
        .order_by(block_height.desc())
        .select(block_height)
        .first::<i32>(conn);

    match res {
        Ok(latest) => Ok(latest as u64),
        Err(e) => match e {
            diesel::NotFound => Ok(0),
            _ => panic!("Error getting latest block {:?}", e),
        },
    }
}

pub fn save(conn: &mut PgConnection, block_height: u64) -> QueryResult<()> {
    diesel::insert_into(perp_event::indexer_processed_block::table)
        .values(&IndexerProcessedBlock {
            // this could be a problem if NEAR exists for more than 68 years
            block_height: block_height as i32,
        })
        .execute(conn)?;

    Ok(())
}
