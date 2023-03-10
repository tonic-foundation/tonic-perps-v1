-- This file should undo anything in `up.sql`
begin
;

alter table
    perp_event.edit_position_event
drop
    column limit_order_id;

alter table
    perp_event.edit_position_event
add
    column trigger_id text default '';

drop table perp_event.remove_limit_order_event;
drop table perp_event.place_limit_order_event;

drop type perp_event.order_type;
drop type perp_event.threshold_type;
drop type perp_event.limit_order_remove_reason;

end;
