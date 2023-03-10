extern crate diesel;

use db::PgPool;
use models::latest_processed_block;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

use anyhow::Result;
use clap::Parser;
use tracing::info;

use config::{init_logging, NearNetwork, Opts, SubCommand};

use futures::StreamExt;
use near_lake_framework::LakeConfigBuilder;

use crate::constants::TARGET;

mod config;
mod constants;
mod db;
mod event;
mod models;
mod schema;
mod tps_counter;
mod worker;

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    init_logging();

    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Run(cli_config) => {
            info!(target: TARGET, "Connecting to DB");
            let db_pool = db::connect();

            let mut conn = db_pool.get().expect("Unable to get connection");
            let latest_processed = latest_processed_block::get(&mut conn).unwrap();
            let starting_block = cli_config.from_blockheight.unwrap_or(latest_processed);
            info!(target: TARGET, "Starting from block {}", starting_block);
            let with_network_config = match cli_config.network {
                NearNetwork::Mainnet => LakeConfigBuilder::default().mainnet(),
                NearNetwork::Testnet => LakeConfigBuilder::default().testnet(),
            };
            let lake_config = with_network_config
                .start_block_height(starting_block)
                .build()
                .expect("Failed to build LakeConfig");
            let (_, stream) = near_lake_framework::streamer(lake_config);

            let wrapped_counter = create_tps_counter();
            let wrapped_counter_copy = wrapped_counter.clone(); // hack
            tokio::spawn(async move {
                let mut log_interval = time::interval(Duration::from_secs(30));
                loop {
                    log_interval.tick().await;
                    let mut counter = wrapped_counter_copy.lock().unwrap();
                    tps_counter::lap_and_log_tps(&mut counter);
                }
            });

            let worker = create_worker(cli_config.contract_id, db_pool);
            let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
                .map(|m| {
                    let mut counter = wrapped_counter.lock().unwrap();
                    counter.add(1);
                    worker.process_message(m)
                })
                .buffer_unordered(1usize);

            while handlers.next().await.is_some() {}
            drop(handlers)
        }
    }

    Ok(())
}

fn create_worker(contract_id: String, pool: PgPool) -> worker::Worker {
    info!(target: TARGET, "Starting worker");
    worker::Worker::new(contract_id, pool)
}

fn create_tps_counter() -> Arc<Mutex<tps_counter::TpsCounter>> {
    let tps_counter = tps_counter::TpsCounter::default();
    let mutex_tps_counter: Mutex<tps_counter::TpsCounter> = Mutex::new(tps_counter);
    Arc::new(mutex_tps_counter)
}
