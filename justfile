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
	cargo watch -x run -w backend

frontend: (yarn-run "start")

yarn-run action:
	cd frontend && yarn run {{action}}

yarn:
	cd frontend && yarn

tunnel:
	@echo "Tunneling local=>server 8000=>8888 3000=>3333"
	@ssh -N -L 8000:localhost:8888 -L 3000:localhost:3333 app@192.168.2.2
