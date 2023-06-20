create type visibility as enum ('public', 'protected', 'private');

create table public_keys
(
    id          serial primary key,

    cert        bytea              not null,
    fingerprint bytea              not null unique,

    -- TODO: maybe users should be validated with a valid email address?
    -- ex: is_validated bool default false not null,
    is_premium  bool default false not null
);

create table pastes
(
    id              serial primary key,
    public_key_id   int          not null
        references public_keys (id) on delete restrict,

    slug            varchar(255) not null unique,
    mime            text         not null,
    visibility      visibility   not null,
    content         bytea        not null,

    created_at      timestamp    not null,
    burn_at         timestamp    not null,
    -- Set the burn_at to now() when read
    burn_after_read boolean      not null
);
