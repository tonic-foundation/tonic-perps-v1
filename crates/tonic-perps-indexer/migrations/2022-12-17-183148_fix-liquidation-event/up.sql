-- Your SQL goes here

alter table
    perp_event.liquidate_position_event
rename
    column collateral_native to collateral_usd;
