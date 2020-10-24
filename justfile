build-schema:
	cd backend && cargo run --features=generate_schema
	just yarn-run codegen

clean-schema:
	find . -type d -name __generated__ -prune -exec echo {} \;

schema:
	cargo watch -s "just build-schema" -w frontend -w backend -i "**/__generated__/**"

backend-build:
	cargo build --release

backend:
	cargo watch -x "run" -w backend -w config.toml

backend-mitm:
	cd backend && cargo watch -x "run --features mitm_proxy" -w . -w ../config.toml

frontend: (yarn-run "start")

yarn-run action:
	cd frontend && yarn run {{action}}

yarn:
	cd frontend && yarn
