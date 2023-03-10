-- This file should undo anything in `up.sql`
-- Your SQL goes here
alter table
    perp_event.edit_position_event
alter
    column liquidator_id set not null; 
