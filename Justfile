_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Run the cli with debug config and environnement
run-cli *ARGS:
	cargo run --bin pgpaste -- --config pgpaste-cli/config.toml {{ARGS}}

# Run the server
run-server *ARGS:
	cargo run --bin pgpaste-server {{ARGS}}

fmt:
	cargo fmt -- --config "group_imports=StdExternalCrate"

# Starts the docker compose file with the provided scope
up:
	docker compose --file docker-compose.local.yaml up -d
# Stops the docker compose file with the provided scope
down:
	docker compose --file docker-compose.local.yaml down
# Builds the docker image with the provided tag
build TAG *ARGS:
	docker build . -t ghcr.io/mrnossiom/pgpaste-server:{{TAG}} {{ARGS}}

# Retrieves the IP address of the local database
local-db-ip:
	@docker inspect -f {{"'{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}'"}} pgpaste-database-1
