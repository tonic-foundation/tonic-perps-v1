-- Your SQL goes here
begin
;

create type perp_event.mint_burn_direction as enum('mint', 'burn');

create table perp_event.lp_mint_burn_event (
    id serial primary key,
    receipt_id text not null,
    account_id text not null,
    block_timestamp timestamp not null,
    direction perp_event.mint_burn_direction not null,
    amount_in text not null,
    amount_out text not null,
    fees text not null,
    fees_usd text not null,
    fees_bps int not null,
    lp_price_usd text not null,
    token_in text not null,
    token_out text not null
);

create index lp_mint_burn_account_id on perp_event.lp_mint_burn_event(account_id);

create index lp_mint_burn_block_timestamp on perp_event.lp_mint_burn_event(block_timestamp desc);

create view perp_event.lp_mint_event as
select
    *
from
    perp_event.lp_mint_burn_event
where
    direction = 'mint';

create view perp_event.lp_burn_event as
select
    *
from
    perp_event.lp_mint_burn_event
where
    direction = 'burn';

-- triggered on new mint/burn event
create function perp_event.handle_new_lp_mint_burn() returns trigger as $body$
declare
    price float;

volume float;

begin
    -- 6 decimal usd to float
    price = util.native_to_decimal(NEW .lp_price_usd, 6);

volume = case
    when NEW .direction = 'mint' then util.native_to_decimal(NEW .amount_out, 18)
    else util.native_to_decimal(NEW .amount_in, 18)
end;

perform perp_event.update_latest_candles(
    NEW .block_timestamp,
    price,
    price,
    price,
    price,
    volume
);

return null;

end;

$body$ language plpgsql;

create trigger update_candles after
insert
    on perp_event.lp_mint_burn_event for each row execute function perp_event.handle_new_lp_mint_burn();

end;