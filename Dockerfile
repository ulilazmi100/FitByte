# Stage 1: Build the application
FROM rust:latest AS builder
WORKDIR /app
COPY . .

# Install dependencies and build the application
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:buster-slim
WORKDIR /app

# Install runtime dependencies (if needed)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/fitbyte_backend .

# Copy the .env file (if needed)
COPY .env .

# Expose the port your application runs on
EXPOSE 8080

# Set the entrypoint to run your application
CMD ["./fitbyte_backend"]