-- This file should undo anything in `up.sql`

alter table
    perp_event.edit_position_event
drop
    column collateral_delta_usd;
