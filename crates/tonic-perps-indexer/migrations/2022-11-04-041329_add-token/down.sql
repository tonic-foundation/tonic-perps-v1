-- This file should undo anything in `up.sql`
-- Your SQL goes here
begin
;

drop view app.token_denomination;

drop table app.token;

end;