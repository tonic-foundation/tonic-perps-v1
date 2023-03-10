-- Your SQL goes here
begin
;

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

alter table
    perp_event.edit_position_event
add
    column trigger_id text default '';

end;