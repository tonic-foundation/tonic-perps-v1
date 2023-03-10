-- Your SQL goes here

alter table
    perp_event.liquidate_position_event
add
    column fees_usd text not null default '0';

alter table
    perp_event.liquidate_position_event
add
    column fees_native text not null default '0';

alter table
    perp_event.liquidate_position_event
alter
    column fees_usd drop default;

alter table
    perp_event.liquidate_position_event
alter
    column fees_native drop default;
