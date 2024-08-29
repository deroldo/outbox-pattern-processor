create table outbox
(
    idempotent_key   uuid        not null,
    partition_key    uuid        not null,
    destinations     jsonb       not null,
    headers          jsonb,
    payload          text        not null,
    created_at       timestamptz not null default now(),
    processing_until timestamptz not null default now(),
    processed_at     timestamptz,
    primary key (idempotent_key)
);

create index index_outbox_by_partition_key on outbox (partition_key);
create index index_outbox_by_created_at_and_processing_until on outbox (created_at, processing_until);
create index index_outbox_by_processed_at on outbox (processed_at);
