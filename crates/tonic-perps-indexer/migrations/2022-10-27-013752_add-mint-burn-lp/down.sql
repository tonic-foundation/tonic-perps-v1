-- This file should undo anything in `up.sql`
begin
;

drop view perp_event.lp_burn_event;

drop view perp_event.lp_mint_event;

drop table perp_event.lp_mint_burn_event;

drop function perp_event.handle_new_lp_mint_burn;

drop type perp_event.mint_burn_direction;

end;