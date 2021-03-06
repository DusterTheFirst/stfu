build-schema-backend:
	cd backend && cargo run --features=generate_schema

build-schema-frontend:
	just yarn-run codegen

build-schema: build-schema-backend build-schema-frontend

clean-schema:
	find . -type d -name __generated__ -prune -exec echo {} \;

schema: clean-schema
	cd backend && cargo watch -s "just build-schema" -w ../frontend -w . -i "**/__generated__/**"

backend-build:
	cargo build --release

backend:
	cd backend && cargo watch -x "run" -i "example.env"

backend-mitm:
	cd backend && cargo watch -x "run --features mitm_proxy" -i "example.env"

frontend: (yarn-run "start")

yarn-run action:
	cd frontend && yarn run {{action}}

yarn:
	cd frontend && yarn
