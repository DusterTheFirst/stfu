# The builder container
FROM rust:latest as builder
WORKDIR /app

# Copy source code over
COPY . .
# Compile the app
RUN cargo build --release

# The app container
FROM debian:buster-slim
LABEL Author="Zachary Kohnen"
WORKDIR /app

# Copy binary to the app
COPY --from=builder /app/target/release/stfu-backend /app/stfu

# Expose the port
EXPOSE 8000

# Run the app
CMD ["./stfu"]
