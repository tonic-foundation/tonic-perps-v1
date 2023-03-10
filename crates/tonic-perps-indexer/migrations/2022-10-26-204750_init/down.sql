-- This file should undo anything in `up.sql`
begin
;

drop schema perp_event cascade;

revoke connect on database postgres
from
    readonly;

drop function util.create_schema_with_reader;

drop schema util cascade;

drop user readonly;

end;