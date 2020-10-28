# The builder container
FROM rust:latest as builder
WORKDIR /app

# Copy source code over
COPY . .
# Compile the app
RUN cargo build --release

# The app container
FROM debian:10
LABEL Author="Zachary Kohnen"
WORKDIR /app

RUN apt update && apt install libssl-dev libz-dev -y

# Copy binary to the app
COPY --from=builder /app/target/release/stfu-backend /app/stfu

# Expose the data
VOLUME [ "/app/data" ]

# Expose the port
EXPOSE 8000

# Run the app
CMD ["./stfu"]