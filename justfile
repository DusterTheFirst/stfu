backend-build-schema:
	cd backend && cargo run --features=generate_schema
	just yarn codegen

backend-build:
	cargo build --release

backend-run: backend-build-schema
	cargo run

backend:
	cargo watch -s "just backend-run" -w backend

frontend:
	cargo watch -s "just backend-build-schema" -w frontend -i "**/__generated__/**"

yarn action:
	cd frontend && yarn run {{action}}

