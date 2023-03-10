begin
;

drop schema app cascade;

drop table perp_event.lp_price_update_event;

drop function perp_event.handle_new_lp_price_update;

drop function perp_event.update_latest_candles;

drop function util.native_to_decimal;

end;