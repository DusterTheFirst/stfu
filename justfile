build-schema:
	cd backend && cargo run --features=generate_schema
	just yarn codegen

clean-schema:
	find . -type d -name __generated__ -prune -exec echo {} \;

schema:
	cargo watch -s "just build-schema" -w frontend -w backend -i "**/__generated__/**"

backend-build:
	cargo build --release

backend:
	cargo watch -x run -w backend

frontend: (yarn "start")

yarn action:
	cd frontend && yarn run {{action}}

