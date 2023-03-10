-- Your SQL goes here

create table perp_event.edit_pool_balance (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    amount_native text not null,
    new_pool_balance_native text not null,
    increase boolean not null,
    asset_id text not null
);

create table perp_event.edit_reserved_amount (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    amount_native text not null,
    new_reserved_amount_native text not null,
    increase boolean not null,
    asset_id text not null
);

create table perp_event.edit_guaranteed_usd (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    amount_usd text not null,
    new_guaranteed_usd text not null,
    increase boolean not null,
    asset_id text not null
);

create table perp_event.edit_fees (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    fee_native text not null,
    fee_usd text not null,
    fee_type text not null,
    new_accumulated_fees_native text not null,
    new_accumulated_fees_usd text not null,
    increase boolean not null,
    asset_id text not null
);

create table perp_event.token_deposit_withdraw (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    amount_native text not null,
    deposit boolean not null,
    method text not null,
    receiver_id text not null,
    asset_id text not null
);
