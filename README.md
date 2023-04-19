# Tonic Perps V1 - NEAR perpetuals trading

Toinc perps is a perpetual exchange built on NEAR.

The contract is deployed by the Tonic team at the following address :
`v1.tonic-perps.near`. It is also accessible through a [web
interface](https://perps.tonic.foundation/trade).

## Features

- Short/Long positions
- Leveraged positions
- Referrals
- Triggers and limit orders
- Fees and rewards

## Documentation

You can find documentation about this contract
[here](https://docs.tonic.foundation/developers/perps-reference).

## Setup

It is recommended to install the [just] utility in order to deploy the
contract. If you do not wish to do so, you can enter the commands from the
[Justfile](https://github.com/tonic-foundation/tonic-perps-v1/blob/master/Justfile)
manually.

You'll also need the NEAR CLI utility. You can install it using :

```bash
npm install -g near-cli
```

You will also have to create an account. You can find more information about
the NEAR blockchain and how to use it [here](https://docs.near.org/).

### 1. Building

If you do not have rust installed, you need to install it :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(this command is from the [official rust install
documentation](https://www.rust-lang.org/learn/get-started)).

Then, you'll need the `wasm32-unknown-unknown` target. To install it, use :

```bash
rustup target add wasm32-unknown-unknown
```

Once that is done, you can build the contract :

```bash
cargo build --target wasm32-unknown-unknown --release -p tonic-perps
```

This command will build the contract for `wasm32-unknown-unknown` in release
mode.

### 2. Deploying

For the puproses of this tutorial, we will deploy to `testnet`, but deploying
to `mainnet` is quite similar.

```bash
near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/near_contract.wasm
```

This command will return a contract address. You can export it using :

```bash
export TONIC_CONTRACT_ID="<the address>"
```

### 3. Initialization

You will also have to set some other environment variables before calling the
contract :

```bash
export ACCOUNT_ID="<your account address>"
```

You should not change the name of this variables, otherwise the `just` commands
will not work.

To actually initialize the contract, run :

```bash
just init-dex
```

You should now have an initialized and deployed contract.

You can find [here](https://github.com/tonic-foundation/tonic-ops) other
add-ons like bots and monitoring that will help you run the contract. The
oracle bot is especially useful if you want to use the contract.

### 4. Usage

One simple way to use it is by deploying the [official web
interface](https://github.com/tonic-foundation/tonic-perps-app).

You can also manually call contract methods using the NEAR CLI. There are also
some `just` shortcuts to call some basic contract functions like
increasing/decreasing positions.

## Setting up the indexer

The indexer is a core part of the project, and needs to be set up in order for
the API and the web UI to work. To do so, you first have to install the
`diesel` CLI :

```bash
cargo install diesel_cli --no-default-features --features postgres
```

You can learn more about `diesel` [here](https://diesel.rs/).

Create a Postgres database and export the URL to the latter :

```bash
export DATABASE_URL=postgres://username:password@host/database
```

Finally, build and run the indexer :

```
cargo run --release -p tonic-perps-indexer -- run -n testnet --contract-id $TONIC_CONTRACT_ID --from-blockheight <block_height>
```

It is recommended to use the block height of the transaction that deployed your
contract for the `--from-blockheight` argument in order to have the full
history of events in your database. Partial data may lead to bugs.

You can also replace `testnet` with `mainnet` if you are running the contract
on `mainnet`.

There is also a `just` shortcut (but it does not run in release mode so it's a
little bit slower) :

```bash
just start-indexer [block_height]
```

**If running from another terminal, do not forget to re-export the
$TONIC_CONTRACT_ID variable.** When using `just`, it is usually recommended to
have `DATABASE_URL`, `TONIC_CONTRACT_ID` and `ACCOUNT_ID` exported.
