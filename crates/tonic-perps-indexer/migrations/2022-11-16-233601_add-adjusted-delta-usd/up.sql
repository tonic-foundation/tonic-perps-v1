-- Your SQL goes here
alter table
    perp_event.edit_position_event
add
    column adjusted_delta_usd text not null default '0';