# run docker build from parent dir which include didkit and ssi
# docker build --tag disn --file ./disn/Dockerfile .

# Builder stage
# We use the latest Rust stable release as base image
FROM rust:1.58.1 AS builder
# Let's switch our working directory to `app` (equivalent to `cd app`) # The `app` folder will be created for us by Docker in case it does not # exist already.
WORKDIR /app/src
# Copy all files from our working environment to our Docker image
COPY . .
WORKDIR /app/src/disn
# Copy didkit and ssi
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim AS runtime
# Install OpenSSL - it is dynamically linked by some of our dependencies 
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app/release
# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/src/disn/target/release/disn disn 
# We need the configuration file at runtime!
COPY ./disn/configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./disn"]