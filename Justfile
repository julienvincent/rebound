default:
    @just --list

migrate:
    cargo run --bin rebound -- migrate --config config.local.json

run-api:
    just migrate
    cargo run --bin rebound -- serve --config config.local.json

run-ui:
    cd interface && just run

cli *args:
    cargo run --bin rebound-cli -- {{ args }}

generate-openapi:
    mkdir -p openapi
    cd api && cargo run -- gen-openapi > ../openapi/api.json

generate-client:
    cd packages/client && just gen
