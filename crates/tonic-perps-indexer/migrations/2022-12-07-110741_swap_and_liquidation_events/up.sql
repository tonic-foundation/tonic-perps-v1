-- Your SQL goes here

create table perp_event.swap_event (
    id serial primary key,
    account_id text not null,
    block_timestamp timestamp not null,
    receipt_id text not null,
    token_in text not null,
    token_out text not null,
    amount_in text not null,
    amount_out text not null,
    fee_bps integer not null,
    fees_usd text not null,
    fees_native text not null
);

create table perp_event.liquidate_position_event (
    id serial primary key,
    block_timestamp timestamp not null,
    receipt_id text not null,
    liquidator_id text not null,
    owner_id text not null,
    position_id text not null,
    collateral_token text not null,
    underlying_token text not null,
    is_long boolean not null,
    size_usd text not null,
    collateral_native text not null,
    reserve_amount_delta_native text not null,
    liquidation_price_usd text not null,
    liquidator_reward_usd text not null,
    liquidator_reward_native text not null
);

drop table perp_event.create_trigger_event;
drop table perp_event.remove_trigger_event;
