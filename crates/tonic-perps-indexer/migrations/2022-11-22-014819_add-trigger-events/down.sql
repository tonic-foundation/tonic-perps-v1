-- This file should undo anything in `up.sql`
begin
;

drop table perp_event.create_trigger_event;

drop table perp_event.remove_trigger_event;

alter table
    perp_event.edit_position_event
drop
    column trigger_id;

end;