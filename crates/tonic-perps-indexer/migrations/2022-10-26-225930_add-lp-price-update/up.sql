-- creates
-- + perp_event.lp_price_update_event table for raw price updates
-- + app schema
-- + app.lp_price_candle_1m for 1 minute candles, every view <1h is based on this
-- + app.lp_price_candle_1h for 1 hour candles, every view >=1h but <1d is based on this
-- + app.lp_price_candle_1d for 1 day candles, everything view >=1d is based on this
-- + trigger to update the candle tables when new price data is received
begin
;

create function util.native_to_decimal(_native text, _decimals int) returns float as $body$ begin
    return _native :: numeric / pow(10, _decimals) :: numeric;

end;

$body$ language plpgsql;

create table perp_event.lp_price_update_event (
    -- internal id
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp default current_timestamp not null,
    -- native on-chain price (usd, 6 decimals)
    price text not null
);

create index receipt_id on perp_event.lp_price_update_event(receipt_id);

create index block_timestamp on perp_event.lp_price_update_event(block_timestamp);

select
    util.create_schema_with_reader('app');

-- this is a summary table that gets updated with a trigger whenever
-- new events are recorded
create table app.lp_price_candle_1m (
    t timestamp not null,
    o float not null,
    l float not null,
    h float not null,
    c float not null,
    v float not null
);

create unique index lp_candle_1m_timestamp on app.lp_price_candle_1m(t);

-- this is a summary table that gets updated with a trigger whenever
-- new events are recorded
create table app.lp_price_candle_5m (
    t timestamp not null,
    o float not null,
    l float not null,
    h float not null,
    c float not null,
    v float not null
);

create unique index lp_candle_5m_timestamp on app.lp_price_candle_5m(t);

-- this is a summary table that gets updated with a trigger whenever
-- new events are recorded
create table app.lp_price_candle_1h (
    t timestamp not null,
    o float not null,
    l float not null,
    h float not null,
    c float not null,
    v float not null
);

create unique index lp_candle_1h_timestamp on app.lp_price_candle_1h(t);

-- this is a summary table that gets updated with a trigger whenever
-- new events are recorded
create table app.lp_price_candle_1d (
    t timestamp not null,
    o float not null,
    l float not null,
    h float not null,
    c float not null,
    v float not null
);

create unique index lp_candle_1d_timestamp on app.lp_price_candle_1d(t);

-- called in some triggers to update the most recent candles
create function perp_event.update_latest_candles(
    -- raw timestamp, no date trunc
    _t timestamp,
    _o float,
    _l float,
    _h float,
    _c float,
    _v float
) returns void as $body$ begin
    insert into
        app.lp_price_candle_1m as cand (t, o, l, h, c, v)
    values
        (date_trunc('minute', _t), _o, _l, _h, _c, _v) on conflict(t) do
    update
    set
        l = least(cand.l, _l),
        h = greatest(cand.h, _h),
        c = _c,
        v = cand.v + _v;

insert into
    app.lp_price_candle_5m as cand (t, o, l, h, c, v)
values
    (
        to_timestamp(
            floor(
                (
                    extract(
                        'epoch'
                        from
                            _t
                    ) / 300
                )
            ) * 300
        ),
        _o,
        _l,
        _h,
        _c,
        _v
    ) on conflict(t) do
update
set
    l = least(cand.l, _l),
    h = greatest(cand.h, _h),
    c = _c,
    v = cand.v + _v;

insert into
    app.lp_price_candle_1h as cand (t, o, l, h, c, v)
values
    (date_trunc('hour', _t), _o, _l, _h, _c, _v) on conflict(t) do
update
set
    l = least(cand.l, _l),
    h = greatest(cand.h, _h),
    c = _c,
    v = cand.v + _v;

insert into
    app.lp_price_candle_1d as cand (t, o, l, h, c, v)
values
    (date_trunc('day', _t), _o, _l, _h, _c, _v) on conflict(t) do
update
set
    l = least(cand.l, _l),
    h = greatest(cand.h, _h),
    c = _c,
    v = cand.v + _v;

end;

$body$ language plpgsql;

-- triggered on new lp price update to update all candles
create function perp_event.handle_new_lp_price_update() returns trigger as $body$
declare
    new_price float;
    block_timestamp timestamp;

begin
    -- 6 decimal usd to float
    new_price = util.native_to_decimal(NEW .price, 6);
    block_timestamp = NEW.block_timestamp;

    perform perp_event.update_latest_candles(
        block_timestamp,
        new_price,
        new_price,
        new_price,
        new_price,
        0 -- no volume reported in this event
    );

return null;

end;

$body$ language plpgsql;

create trigger update_candles after
insert
    on perp_event.lp_price_update_event for each row execute function perp_event.handle_new_lp_price_update();

end;