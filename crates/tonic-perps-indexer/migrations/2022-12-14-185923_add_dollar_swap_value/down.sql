-- This file should undo anything in `up.sql`

alter table
    perp_event.swap_event
rename
    column amount_in_native to amount_in;

alter table
    perp_event.swap_event
rename
    column amount_out_native to amount_out;

alter table
    perp_event.swap_event
drop
    column amount_in_usd;

alter table
    perp_event.swap_event
drop
    column amount_out_usd;
