-- This file should undo anything in `up.sql`

alter table
    perp_event.liquidate_position_event
drop
    column fees_usd;

alter table
    perp_event.liquidate_position_event
drop
    column fees_native;
