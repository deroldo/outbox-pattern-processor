# Outbox Pattern Processor

A application to make easier to dispatch your outbox-pattern data from database to SQS, SNS or HTTP and HTTPS gateway.

* **Simple**: Your application only need to write into `outbox` table.
* **Scalable**: It's possible to run more than one instance to increase performance without lose order.

[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/deroldo/outbox-pattern-processor/blob/main/LICENSE

## Required

[Database configuration](./database/README.md)

## How to use

[Use as library](./lib/README.md)

[Docker image](./worker/README.md)

## License
This project is licensed under the MIT license.