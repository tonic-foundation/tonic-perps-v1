-- basic things (create user)
begin
;

create user readonly with password 'readonly';

grant connect on database postgres to readonly;

create schema util;

-- create a schema and grant select on all tables to the
-- given user (defaults to the readonly user)
create function util.create_schema_with_reader(_schema text, _reader text default 'readonly') returns void as $$
begin
    execute format('create schema %s', _schema);
    execute format('grant usage on schema %s to %s', _schema, _reader);
    execute format('alter default privileges in schema %s
                    grant select on tables to %s', _schema, _reader);
end;
$$ language plpgsql;

do $$ begin
 perform util.create_schema_with_reader('perp_event');
end $$;

create table perp_event.indexer_processed_block (
    block_height integer primary key,
    processed_at timestamp default current_timestamp
);

end;