-- Your SQL goes here
begin
;

alter table
    perp_event.edit_position_event
add
    column limit_order_id text default '';

alter table
    perp_event.edit_position_event
drop
    column trigger_id;

create table perp_event.place_limit_order_event (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    account_id text not null,
    limit_order_id text not null,
    collateral_token text not null,
    underlying_token text not null,
    order_type text not null,
    threshold_type text not null,
    collateral_delta_usd text not null,
    attached_collateral_native text not null,
    size_delta_usd text not null,
    price_usd text not null,
    expiry timestamp not null,
    is_long boolean not null
);

create table perp_event.remove_limit_order_event (
    id serial primary key,
    account_id text not null,
    block_timestamp timestamp not null,
    receipt_id text not null,
    underlying_token text not null,
    limit_order_id text not null,
    reason text not null,
    liquidator_id text
);

end;
