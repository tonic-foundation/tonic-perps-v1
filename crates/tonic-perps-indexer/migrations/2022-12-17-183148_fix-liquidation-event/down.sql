-- This file should undo anything in `up.sql`

alter table
    perp_event.liquidate_position_event
rename
    column collateral_usd to collateral_native;
