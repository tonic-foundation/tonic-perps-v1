use near_sdk::json_types::U128;
use near_sdk::log;

use crate::{
    emit_event, env, near_bindgen, ratio, AccountId, AssetId, Contract, Deserialize, EventType,
    LpPriceUpdateEvent, OracleUpdateEvent, Serialize, VContract, VContractExt, BPS_DIVISOR,
    DOLLAR_DENOMINATION,
};

use crate::switchboard::{switchboard_contract, Ix, TGAS};

pub type SwitchboardAddress = [u8; 32];

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateIndexPriceRequest {
    pub asset_id: String,
    pub price: U128,
    pub spread: Option<u16>,
}

pub fn switchboard_id() -> AccountId {
    if cfg!(feature = "mainnet") {
        sbv2_near::SWITCHBOARD_V2_MAINNET
    } else {
        sbv2_near::SWITCHBOARD_V2_TESTNET
    }
    .parse()
    .unwrap()
}

impl Contract {
    /// Used to update the index price from keeper script
    fn update_index_price(&mut self, reqs: Vec<UpdateIndexPriceRequest>) {
        let time = env::block_timestamp_ms();
        for req in reqs {
            let asset_id: AssetId = req.asset_id.into();
            if self.assets.get(&asset_id).is_some() {
                let mut asset = self.assets.unwrap(&asset_id);
                if let Some(max_change_bps) = asset.max_price_change_bps {
                    let price_delta = asset.price.abs_diff(req.price.0);
                    let price_increased = asset.price < req.price.0;

                    let max_change_per_second = ratio(asset.price, max_change_bps, BPS_DIVISOR);

                    let max_change = max_change_per_second
                        * std::time::Duration::from_millis(time - asset.last_change_timestamp_ms)
                            .as_secs() as u128;

                    if asset.price != 0 && max_change < price_delta {
                        if price_increased {
                            asset.price += max_change;
                        } else {
                            asset.price -= max_change;
                        }
                    } else {
                        asset.price = req.price.0;
                    }
                } else {
                    asset.price = req.price.0;
                }
                if let Some(spread) = req.spread {
                    asset.spread_bps = spread;
                }
                asset.last_change_timestamp_ms = time;
                emit_event(EventType::OracleUpdate(OracleUpdateEvent {
                    asset_id: asset_id.into_string(),
                    price: U128(asset.price),
                    spread_bps: req.spread.unwrap_or(0),
                    source: "tonic".to_string(),
                }));
                self.set_asset(&asset_id.clone(), asset);
            };
        }

        for (asset_id, mut asset) in self.get_assets() {
            self.update_cumulative_funding_rate(&mut asset);
            self.set_asset(&asset_id.clone(), asset);
        }

        // Won't have any supply upon initialization
        if self.lp_token.total_supply > 0 {
            emit_event(EventType::LpPriceUpdate(LpPriceUpdateEvent {
                price: U128(self.lp_price()),
            }));
        }
    }
}

#[near_bindgen]
impl VContract {
    /// Admin manual price updater.
    pub fn update_index_price(&mut self, reqs: Vec<UpdateIndexPriceRequest>) {
        let contract = self.contract_mut();
        contract.assert_price_oracle();
        contract.update_index_price(reqs);
    }

    /// Initiates query to Switchboard which will update asset prices in callbacks
    pub fn query_switchboard(&mut self) {
        let contract = self.contract();
        contract.assert_owner();
        let gas = near_sdk::Gas(5 * TGAS);

        for asset in &contract.assets.0 {
            if let Some(address) = asset.1.switchboard_aggregator_address {
                let ix = Ix::new(address);

                switchboard_contract::ext(switchboard_id())
                    .with_static_gas(gas)
                    .aggregator_read(ix)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(near_sdk::Gas(5 * TGAS))
                            .query_switchboard_callback(asset.0),
                    );
            }
        }
    }

    #[private]
    pub fn query_switchboard_callback(
        &mut self,
        #[callback_result] call_result: Result<sbv2_near::AggregatorRound, near_sdk::PromiseError>,
        asset_id: &AssetId,
    ) {
        if let Ok(res) = call_result {
            let contract = self.contract_mut();

            if res.result.mantissa < 0 {
                env::panic_str("Price of asset cannot be negative");
            }
            let mantissa = res.result.mantissa as u128;
            let scale = res.result.scale;

            let price: u128 = ratio(mantissa, DOLLAR_DENOMINATION, 10u128.pow(scale));

            contract.update_index_price(vec![UpdateIndexPriceRequest {
                asset_id: asset_id.into_string(),
                price: U128(price),
                spread: None,
            }]);
        } else {
            log!("Error in Switchboard callback");
        }
    }
}
