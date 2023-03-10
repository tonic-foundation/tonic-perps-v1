set dotenv-load
set shell := ["bash", "-c"]

contract_id := `echo "$TONIC_CONTRACT_ID"`
account_id := `echo "$ACCOUNT_ID"`
database_url := `echo "$DATABASE_URL"`

# Show help
default:
    just --list

# Run clippy
lint:
    cargo clippy

# Build release
build-release:
    rustup target add wasm32-unknown-unknown && cargo build --target wasm32-unknown-unknown --release

# Build release
build-contract:
    #!/bin/bash
    set -eu

    reset() {
        perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' crates/tonic-perps/Cargo.toml
    }

    hack_skip_rlib() {
        perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' crates/tonic-perps/Cargo.toml
    }

    trap reset EXIT;

    # Don't build rlib in production (saves about 1.4MB)
    hack_skip_rlib

    cargo build --target wasm32-unknown-unknown --release -p tonic-perps

# Mint test tokens
mint-tokens:
    near call fake-btc.testnet ft_mint '{"receiver_id": "{{account_id}}", "amount": "10000000"}' --accountId {{account_id}}
    near call fake-eth.testnet ft_mint '{"receiver_id": "{{account_id}}", "amount": "1000000000000000000"}' --accountId {{account_id}}
    near call fake-usdc.testnet ft_mint '{"receiver_id": "{{account_id}}", "amount": "1000000000"}' --accountId {{account_id}}
    near call fake-usdt.testnet ft_mint '{"receiver_id": "{{account_id}}", "amount": "1000000000"}' --accountId {{account_id}}

init-dex:
    near call {{contract_id}} new --accountId {{account_id}}
    near call {{contract_id}} add_price_oracle '{"account_id": "'{{account_id}}'"}' --accountId {{account_id}}

    near call {{contract_id}} add_asset '{"asset_id": "near", "decimals": 24, "stable": false, "weight": 20}' --accountId {{account_id}}
    near call {{contract_id}} add_asset '{"asset_id": "fake-usdc.testnet", "decimals": 6, "stable": true, "weight": 20}' --accountId {{account_id}}
    near call {{contract_id}} add_asset '{"asset_id": "fake-usdt.testnet", "decimals": 6, "stable": true, "weight": 20}' --accountId {{account_id}}
    near call {{contract_id}} add_asset '{"asset_id": "fake-btc.testnet", "decimals": 8, "stable": false, "weight": 20}' --accountId {{account_id}}
    near call {{contract_id}} add_asset '{"asset_id": "fake-eth.testnet", "decimals": 18, "stable": false, "weight": 20}' --accountId {{account_id}}

    near call {{contract_id}} set_default_stablecoin '{"asset_id": "fake-usdt.testnet"}' --accountId {{account_id}}

    near call {{contract_id}} update_index_price '{"reqs": [{"asset_id": "near", "price": "2000000"},{"asset_id": "fake-usdc.testnet", "price": "1000000"},{"asset_id": "fake-usdt.testnet", "price": "1000000"}, {"asset_id": "fake-eth.testnet", "price": "1000000000"}, {"asset_id": "fake-btc.testnet", "price": "15000000000"}]}' --accountId {{account_id}}

    near call fake-btc.testnet ft_mint '{"receiver_id": "{{contract_id}}", "amount": "10000000"}' --accountId {{account_id}}
    near call fake-eth.testnet ft_mint '{"receiver_id": "{{contract_id}}", "amount": "1000000000000000000"}' --accountId {{account_id}}
    near call fake-usdc.testnet ft_mint '{"receiver_id": "{{contract_id}}", "amount": "1000000000"}' --accountId {{account_id}}
    near call fake-usdt.testnet ft_mint '{"receiver_id": "{{contract_id}}", "amount": "1000000000"}' --accountId {{account_id}}

    near call {{contract_id}} set_state '{"state": "Running"}' --accountId {{account_id}}

    near call {{contract_id}} mint_lp_near '{}' --deposit 10 --accountId {{account_id}}
    near call fake-usdc.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "1000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-usdt.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "1000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-eth.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "100000000000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-btc.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "10000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call {{contract_id}} set_shortable '{"asset_id": "near", "shortable": true}' --accountId {{account_id}}
    near call {{contract_id}} set_shortable '{"asset_id": "fake-btc.testnet", "shortable": true}' --accountId {{account_id}}
    near call {{contract_id}} set_shortable '{"asset_id": "fake-eth.testnet", "shortable": true}' --accountId {{account_id}}

mint-tlp:
    near call {{contract_id}} mint_lp_near '{}' --deposit 10 --accountId {{account_id}}
    near call fake-usdc.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "1000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-usdt.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "1000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-eth.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "100000000000000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}
    near call fake-btc.testnet ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "10000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}

# Update index price
update-price asset_id price:
    near call {{contract_id}} update_index_price '{"reqs": [{"asset_id": "{{asset_id}}", "price": "{{price}}"}]}' --accountId {{account_id}}

# Increase position near
increase-position-near size_delta collateral_delta:
    near call {{contract_id}} increase_position '{"params": {"underlying_id": "near", "size_delta": "{{size_delta}}", "is_long": true}}' --deposit {{collateral_delta}} --accountId {{account_id}}

# Increase position ft
increase-position-ft collateral_id underlying_id size_delta collateral_delta is_long:
    near call {{collateral_id}} ft_transfer_call '{"receiver_id": "{{contract_id}}", "amount": "{{collateral_delta}}", "msg": "{\"action\": \"IncreasePosition\", \"params\": {\"underlying_id\": \"{{underlying_id}}\", \"size_delta\": \"{{size_delta}}\", \"is_long\": {{is_long}}}}"}' --depositYocto 1 --gas 300000000000000 --accountId {{account_id}}

# Decrease position
decrease-position position_id size_delta collateral_delta:
    near call {{contract_id}} decrease_position '{"params": {"size_delta": "{{size_delta}}", "position_id": "{{position_id}}", "collateral_delta": "{{collateral_delta}}"}}' --accountId {{account_id}}

# Add limit order
add-limit-order-near price order_type size_delta collateral_delta:
    near call {{contract_id}} add_limit_order '{"params":{"price": "{{price}}", "size_delta": "{{size_delta}}", "underlying_id": "near", "is_long": true, "order_type": "{{order_type}}"}}' --accountId {{account_id}} --deposit {{collateral_delta}}

# Remove limit order
remove-limit-order underlying_id limit_order_id:
    near call {{contract_id}} remove_limit_order '{"limit_order_id":"{{limit_order_id}}", "asset_id": "{{underlying_id}}"}' --accountId {{account_id}}

# Execute limit order
execute-limit-order underlying_id limit_order_id:
    near call {{contract_id}} execute_limit_order '{"limit_order_id":"{{limit_order_id}}", "asset_id": "{{underlying_id}}"}' --accountId {{account_id}} --gas 300000000000000

# Start indexer
start-indexer block_height=("0"):
    DATABASE_URL='{{database_url}}' cargo run -p tonic-perps-indexer -- run -n testnet --contract-id {{contract_id}} --from-blockheight {{block_height}}
