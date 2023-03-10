-- Your SQL goes her
begin
;

create type perp_event.edit_position_direction as enum('increase', 'decrease');

create type perp_event.edit_position_state as enum('created', 'closed', 'open');

create table perp_event.edit_position_event (
    id serial primary key,
    receipt_id text not null,
    account_id text not null,
    block_timestamp timestamp not null,
    position_id text not null,
    direction perp_event.edit_position_direction not null,
    state perp_event.edit_position_state not null,
    collateral_token text not null,
    underlying_token text not null,
    collateral_delta_native text not null,
    size_delta_usd text not null,
    new_size_usd text not null,
    is_long boolean not null,
    price_usd text not null,
    total_fee_usd text not null,
    margin_fee_usd text not null,
    position_fee_usd text not null,
    total_fee_native text not null,
    margin_fee_native text not null,
    position_fee_native text not null,
    usd_out text not null,
    realized_pnl_to_date_usd text not null,
    referral_code text default ''
);

create index edit_position_account_id on perp_event.edit_position_event(account_id);

create index edit_position_block_timestamp on perp_event.edit_position_event(block_timestamp desc);

create index edit_position_position_id on perp_event.edit_position_event(position_id);

end;