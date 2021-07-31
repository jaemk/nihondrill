begin;

create table nd.maps (
    id        int8        not null primary key default nd.id_gen(),
    created   timestamptz not null             default now(),
    modified  timestamptz not null             default now(),
    deleted   boolean     not null             default false,
    english   text        not null,
    japanese  text        not null
);
create index map_english on nd.maps using gin (english gin_trgm_ops);
create index map_japanese on nd.maps using gin (japanese gin_trgm_ops);

create table nd.prompt_enum (
    id text not null primary key
);
insert into nd.prompt_enum (id)
values ('english'),
       ('japanese');

create table nd.questions (
    id        int8        not null primary key default nd.id_gen(),
    created   timestamptz not null             default now(),
    modified  timestamptz not null             default now(),
    user_id   int8        not null references nd.users(id),
    map_id    int8        not null,
    prompt    text        not null references nd.prompt_enum(id),
    answer    text,
    answered  timestamptz,
    correct   boolean
);
create index question_user_id on nd.questions(map_id);
create index question_map_id on nd.questions(map_id);
create index question_prompt on nd.questions(prompt);
create index question_created on nd.questions(created);
create index question_answered on nd.questions(answered);
create index question_answer on nd.questions using gin (answer gin_trgm_ops);

commit;
