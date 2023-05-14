container-name := "pgpaste-server"

_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Run the server
run-cli *ARGS:
	cargo run -q --bin pgpaste-cli -- --config pgpaste-cli/config.toml {{ARGS}}

# Run the server
run-server:
	cargo run --bin pgpaste-server

# Starts the docker compose file with the provided scope
up SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yml up -d
# Stops the docker compose file with the provided scope
down SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yml down
# Builds the docker image with the provided tag
build TAG:
	docker build . -t ghcr.io/mrnossiom/{{container-name}}:{{TAG}}

# Retrieves the IP address of the local database
local-db-ip:
	@docker inspect -f {{"'{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}'"}} {{container-name}}-database-1