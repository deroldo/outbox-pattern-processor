# code format
fmt:
	cargo fmt

# code lint
lint:
	cargo clippy

# code test
test:
	docker-compose up -d
	make wait_test_stack
	cargo nextest run --test-threads=1
	docker-compose down

wait_test_stack:
	@while [ $$(docker-compose ps | grep postgres | grep healthy | wc -l) -eq 0 ]; do echo "waiting for database to be healthy..."; sleep 1; done
	@while [ $$(docker-compose ps | grep localstack | grep healthy | wc -l) -eq 0 ]; do echo "waiting for localstack to be healthy..."; sleep 1; done

# code format, lint and test
validate: fmt lint test

# run migration on docker compose database
local_migration:
	cargo sqlx migrate run --source database/migrations --database-url postgres://local:local@localhost:5432/local