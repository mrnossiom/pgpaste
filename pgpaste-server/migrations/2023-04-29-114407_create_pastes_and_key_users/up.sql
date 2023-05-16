CREATE TYPE visibility AS ENUM ('public', 'protected', 'private');

CREATE TABLE public_keys
(
    id          serial PRIMARY KEY,
    fingerprint bytea NOT NULL UNIQUE,
    key         bytea NOT NULL
);

CREATE TABLE pastes
(
    id         serial PRIMARY KEY,
    /*public_key_id int          NOT NULL
        REFERENCES public_keys (id) ON DELETE RESTRICT,*/

    slug       varchar(255) NOT NULL UNIQUE,
    visibility visibility   NOT NULL,
    content    bytea        NOT NULL,

    created_at timestamp    NOT NULL,
    burn_at    timestamp    NOT NULL
);
