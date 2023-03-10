use tonic_perps_sdk::prelude::Base58VecU8;

pub fn base58_encode(data: &Base58VecU8) -> String {
    bs58::encode(&data.0).into_string()
}
