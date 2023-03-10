-- Your SQL goes here
begin
;

create table app.token (
    id varchar(64) primary key,
    name text not null,
    symbol text not null,
    decimals smallint not null,
    -- spec text default 'ft-1.0.0',
    -- icon text,
    -- reference text,
    -- reference_hash text,
    created_at timestamp default current_timestamp
);

create view app.token_denomination as (
    select
        *,
        pow(10, decimals) denomination
    from
        app.token
);

end;