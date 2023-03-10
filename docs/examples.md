# Contract Usage Examples

```
export TONIC_CONTRACT=dev-1664857789063-85098841650832
```

# Example Tokens for your use

```
near call fake-usdc.testnet ft_mint '{"receiver_id": "'$ACCOUNT_ID'", "amount": "100000000"}' --accountId $ACCOUNT_ID
near call fake-btc.testnet ft_mint '{"receiver_id": "'$ACCOUNT_ID'", "amount": "100000000"}' --accountId $ACCOUNT_ID
near call fake-eth.testnet ft_mint '{"receiver_id": "'$ACCOUNT_ID'", "amount": "100000000"}' --accountId $ACCOUNT_ID
```

# Initialize Contract

```
near call $TONIC_CONTRACT new --accountId $ACCOUNT_ID
```

# Add Assets

```
near call $TONIC_CONTRACT add_asset '{"asset_id": "near", "decimals": 24, "stable": false, "weight": 25}' --accountId $ACCOUNT_ID
near call $TONIC_CONTRACT add_asset '{"asset_id": "fake-usdc.testnet", "decimals": 6, "stable": true, "weight": 25}' --accountId $ACCOUNT_ID
```

# Mint LP Token

## Storage Deposit

```
near call $TONIC_CONTRACT storage_deposit '{}' --deposit 1 --accountId $ACCOUNT_ID
```

## NEAR Mint

```
near call $TONIC_CONTRACT mint_lp_near '{}' --deposit 1 --accountId $ACCOUNT_ID
```

## FT Mint

```
near call fake-usdc.testnet ft_transfer_call '{"receiver_id": "'$TONIC_CONTRACT'", "amount": "100000000", "msg": "{\"action\": \"MintLp\", \"params\": {}}"}' --depositYocto 1 --gas 300000000000000 --accountId $ACCOUNT_ID
```

# Burn LP Token

```
near call $TONIC_CONTRACT burn_lp_token '{"amount": "499950000000000000", "asset_out": "NEAR"}' --accountId $ACCOUNT_ID
```

# Swap

## Swap NEAR -> FT

```
near call $TONIC_CONTRACT swap_near '{"token_out": "fake-usdc.testnet"}' --deposit 0.1 --accountId tust.testnet
```

## Swap FT -> NEAR

```
near call fake-btc.testnet ft_transfer_call '{"receiver_id": "'$TONIC_CONTRACT'", "amount": "10000", "msg": "{\"action\": \"Swap\", \"params\": {\"output_token\": \"fake-usdc.testnet\"}}"}' --depositYocto 1 --gas 300000000000000 --accountId $ACCOUNT_ID
```

# Perps

## Open/Increase Position

### NEAR

```
near call $TONIC_CONTRACT increase_position '{"params": {"underlying_id": "near", "size_delta": "100000000", "is_long": true}}' --deposit 10 --accountId $ACCOUNT_ID
```

### FT

```
near call fake-btc.testnet ft_transfer_call '{"receiver_id": "'$TONIC_CONTRACT'", "amount": "1000000", "msg": "{\"action\": \"IncreasePosition\", \"params\": {\"underlying_id\": \"test\", \"size_delta\": \"1000000000\", \"is_long\": true}}"}' --depositYocto 1 --gas 300000000000000 --accountId $ACCOUNT_ID
```

## Decrease Position

## Deposit Collateral

### NEAR

### FT

## Withdraw Collateral

## Triggers

# Oracle

## Add Price Oracle

```
near call $TONIC_CONTRACT add_price_oracle '{"account_id": "'$ACCOUNT_ID'"}' --accountId $ACCOUNT_ID
```

## Update Index Price

```
near call $TONIC_CONTRACT update_index_price '{"reqs": [{"asset_id": "near", "price": "3500000"}]}' --accountId $ACCOUNT_ID
near call $TONIC_CONTRACT update_index_price '{"reqs": [{"asset_id": "fake-usdc.testnet", "price": "1000000"}]}' --accountId $ACCOUNT_ID
```

# View

## Asset Info

```
near view $TONIC_CONTRACT get_asset_info '{"asset": "near"}'
```

## Open Positions

```
near view $TONIC_CONTRACT get_positions '{"account_id": "'$ACCOUNT_ID'"}'
```

## Open Orders

## LP Token Balance

```
near view $TONIC_CONTRACT ft_balance_of '{"account_id": "'$ACCOUNT_ID'"}'
```

## LP Token Price

```
near view $TONIC_CONTRACT get_lp_price '{}'
```

## Get Position ID

# Init Steps

- init contract
- add assets
- add price oracle
- set prices for assets
- mint LP token
