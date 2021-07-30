begin;

create table nd.users
(
    id        int8        not null primary key default nd.id_gen(),
    created   timestamptz not null             default now(),
    modified  timestamptz not null             default now(),
    name      text        not null,
    email     text        not null
);
create unique index user_email on nd.users (email);

create table nd.auth_tokens
(
    id        int8        not null primary key default nd.id_gen(),
    created   timestamptz not null             default now(),
    modified  timestamptz not null             default now(),
    expires   timestamptz not null,
    user_id   int8        not null references  nd.users (id),
    signature text        not null
);
create index auth_token_user on nd.auth_tokens (user_id);
create index auth_token_sig on nd.auth_tokens (signature);

commit;

