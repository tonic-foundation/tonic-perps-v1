-- Your SQL goes here

alter table
    perp_event.edit_position_event
add
    column liquidator_id text not null default '';

alter table
    perp_event.edit_position_event
alter
    column liquidator_id drop default;
