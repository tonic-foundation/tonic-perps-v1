-- Your SQL goes here

alter table
    perp_event.swap_event
rename
    column amount_in to amount_in_native;

alter table
    perp_event.swap_event
rename
    column amount_out to amount_out_native;

alter table
    perp_event.swap_event
add
    column amount_in_usd text not null default '0';

alter table
    perp_event.swap_event
add
    column amount_out_usd text not null default '0';

alter table
    perp_event.swap_event
alter
    column amount_in_usd drop default;

alter table
    perp_event.swap_event
alter
    column amount_out_usd drop default;
