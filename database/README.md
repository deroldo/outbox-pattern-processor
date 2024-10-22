# Outbox Pattern Processor - Rust library

A application to make easier to dispatch your outbox-pattern data from database to SQS, SNS or HTTP and HTTPS gateway.

* **Simple**: Your application only need to write into `outbox` table.
* **Scalable**: It's possible to run more than one instance to increase performance without lose order.

[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/deroldo/outbox-pattern-processor/blob/main/LICENSE

## Database compatibility

### Postgres

#### Required table

```sql
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

create table outbox_lock
(
    partition_key    uuid        not null,
    lock_id          uuid        not null,
    processed_at     timestamptz,
    processing_until timestamptz not null default now() + ('30 seconds')::interval
);

create table outbox_cleaner_schedule
(
    cron_expression varchar(50) not null,
    last_execution timestamptz not null default now()
);
```

#### Required indexes
```sql
create index idx_outbox_by_partition_key on outbox (partition_key);
create index idx_outbox_by_process_after on outbox (process_after);
create index idx_outbox_by_processed_at on outbox (processed_at);
create index idx_outbox_by_partition_key_and_process_after on outbox (partition_key, process_after);
create index idx_outbox_by_part_key_and_proc_after_and_attempt_and_proc_at on outbox (partition_key, process_after, attempts) where processed_at is null;

create unique index unq_outbox_lock_by_partition_key on outbox_lock (partition_key) where processed_at is null;
create index idx_outbox_lock_by_lock_id on outbox_lock (lock_id);
create index idx_outbox_lock_by_processing_until on outbox_lock (processing_until) where processed_at is null;
create index idx_outbox_lock_by_processed_at on outbox_lock (processed_at);
```

### Tabla outbox - columns details

#### idempotent_key

- Primary Key
- UUID format (preferably UUIDv7)

###### Example: `01919ea7-2049-7f94-a77e-75f7fecbb28e`

#### partition_key

- UUID format (preferably UUIDv7)
- Used to ensure the order from outbox events provided by the same "origin"

###### Example: `01919ea7-85ac-755e-a77d-a1d90438b260`

#### destinations

- JSON Array of destination kinds
- Used to dispatch outbox events

SQS
```json
{
  "queue_url": "<AWS queue url format>"
}
```

SNS
```json
{
  "topic_arn": "<AWS topic arn format>"
}
```

HTTP
```json
{
  "url": "<HTTP url format>",
  "headers": {
    "key": "value"
  }, // optional
  "method": "<POST|PUT|PATCH>" // optional - default POST
}
```

###### Simple example:
```json
[
  {
    "queue_url": "http://sqs.us-east-1.localhost.localstack.cloud:4566/000000000000/queue"
  }
]
```

###### Full example:
```json
[
  {
    "queue_url": "http://sqs.us-east-1.localhost.localstack.cloud:4566/000000000000/queue"
  },
  {
    "topic_arn": "arn:aws:sns:us-east-1:000000000000:topic"
  },
  {
    "url": "https://my-domain.com/first",
    "headers": {
      "Content-Type": "application/json"
    },
    "method": "PUT"
  },
  {
    "url": "http://my-domain.com/second",
    "method": "PATCH"
  },
  {
    "url": "https://my-domain.com/third",
    "headers": {
      "Content-Type": "application/json"
    }
  },
  {
    "url": "http://my-domain.com/fourth"
  }
]
```

#### headers

- JSON Object
- Used to add custom headers to destination
- `x-idempotent-key` is always sent

###### Example:
```json
{
  "x-event-type": "creation"
}
```

#### payload

- Outbox event content

###### Example: `Some message, it can be a stringify JSON too`

### Tabla outbox_cleaner_schedule - columns details

#### cron_expression

- Outbox cleaner cron expression

###### Example: `0 0 */3 * * *` -- every 3 hours


## License
This project is licensed under the MIT license.