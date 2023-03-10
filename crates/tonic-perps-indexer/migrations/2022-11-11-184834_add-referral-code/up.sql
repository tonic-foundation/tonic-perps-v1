-- Your SQL goes here
begin
;

create table perp_event.create_referral_code_event (
    id serial primary key,
    receipt_id text not null,
    block_timestamp timestamp not null,
    -- owner
    account_id text not null,
    code text not null,
    created_at timestamp default current_timestamp
);

create index referral_code_account_id on perp_event.create_referral_code_event(account_id);

create index referral_code_code on perp_event.create_referral_code_event(code);

end;