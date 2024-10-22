create table outbox
(
    idempotent_key   uuid        not null,
    partition_key    uuid        not null,
    destinations     jsonb       not null,
    headers          jsonb,
    payload          text        not null,
    attempts         int         not null default 0,
    created_at       timestamptz not null default now(),
    process_after    timestamptz not null default now(),
    processed_at     timestamptz,
    primary key (idempotent_key)
);

create index idx_outbox_by_partition_key on outbox (partition_key);
create index idx_outbox_by_process_after on outbox (process_after);
create index idx_outbox_by_processed_at on outbox (processed_at);
create index idx_outbox_by_partition_key_and_process_after on outbox (partition_key, process_after);
create index idx_outbox_by_part_key_and_proc_after_and_attempt_and_proc_at on outbox (partition_key, process_after, attempts) where processed_at is null;

create table outbox_lock
(
    partition_key    uuid        not null,
    lock_id          uuid        not null,
    processed_at     timestamptz,
    processing_until timestamptz not null default now() + ('30 seconds')::interval
);

create unique index unq_outbox_lock_by_partition_key on outbox_lock (partition_key) where processed_at is null;
create index idx_outbox_lock_by_lock_id on outbox_lock (lock_id);
create index idx_outbox_lock_by_processing_until on outbox_lock (processing_until) where processed_at is null;
create index idx_outbox_lock_by_processed_at on outbox_lock (processed_at);
create index idx_outbox_by_partition_key_and_processing_until on outbox_lock (partition_key, processing_until);

create table outbox_cleaner_schedule
(
    cron_expression varchar(50) not null,
    last_execution timestamptz not null default now()
);