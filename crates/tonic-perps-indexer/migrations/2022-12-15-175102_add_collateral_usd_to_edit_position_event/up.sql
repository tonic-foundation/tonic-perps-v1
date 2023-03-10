-- Your SQL goes here

alter table
    perp_event.edit_position_event
add
    column collateral_delta_usd text not null default '';

alter table
    perp_event.edit_position_event
alter
    column collateral_delta_usd drop default;
