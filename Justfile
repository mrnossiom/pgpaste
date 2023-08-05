_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Run the cli with debug config and environnement
cli *ARGS:
	cargo run --bin pgpaste -- --config pgpaste-cli/config.toml {{ARGS}}

# Run the server
server:
	cargo leptos watch

test: test-wasm
	cargo test

test-wasm:
	#!/bin/dash
	cd pgpaste-app
	wasm-pack test --firefox  --headless -- --features __test
	# or with the runner configured in `.cargo/config.toml`
	# GECKODRIVER=/path/to/geckodriver cargo test --target wasm32-unknown-unknown --features __test

# Starts the docker compose file with the provided scope
up SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yaml up -d
# Stops the docker compose file with the provided scope
down SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yaml down
# Builds the docker image with the provided tag
build TAG *ARGS:
	docker build . -t ghcr.io/mrnossiom/pgpaste-server:{{TAG}} {{ARGS}}

# Retrieves the IP address of the local database
local-db-ip:
	@docker inspect -f {{"'{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}'"}} pgpaste-database-1