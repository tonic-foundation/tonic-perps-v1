-- This file should undo anything in `up.sql`

drop table perp_event.swap_event;
drop table perp_event.liquidate_position_event;

create table perp_event.create_trigger_event (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    trigger_id text not null,
    position_id text not null,
    account_id text not null,
    price_usd text not null,
    collateral_delta_native text not null,
    size_delta_usd text not null,
    trigger_type text not null,
    price_change_type text not null
);

create table perp_event.remove_trigger_event (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    trigger_id text not null,
    position_id text not null,
    account_id text not null
);
