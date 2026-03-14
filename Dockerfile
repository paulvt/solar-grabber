# Using multistage build:
#   https://docs.docker.com/develop/develop-images/multistage-build/
#   https://whitfin.io/speeding-up-rust-docker-builds/

##########################  BUILD IMAGE  ##########################
# Rust build image to build Solar Grabber's statically compiled binary
FROM docker.io/rust:1 as builder

# Build the dependencies first
RUN USER=root cargo new --bin /usr/src/solar-grabber
WORKDIR /usr/src/solar-grabber
COPY ./Cargo.* ./
RUN sed -i -e 's/^build =/#build =/' Cargo.toml && \
      cargo build --release && \
      rm src/*.rs

# Add the real project files from current folder
COPY ./Cargo.toml ./
COPY . ./

# Build the actual binary from the copied local files
RUN rm ./target/release/deps/solar_grabber* && \
      cargo build --release

########################## RUNTIME IMAGE ##########################
# Create new stage with a minimal image for the actual runtime image/container
FROM docker.io/debian:trixie-slim

# Install adduser and CA certificates
RUN apt-get update && \
      apt-get upgrade -y && \
      apt-get install -y --no-install-recommends adduser ca-certificates && \
      rm -rf /var/lib/apt/lists/*

# Copy the binary from the "builder" stage to the current stage
RUN adduser --system --disabled-login --home /app --gecos "" --shell /bin/bash solar-grabber
COPY --from=builder /usr/src/solar-grabber/target/release/solar-grabber /app

# Standard port on which Rocket launches
EXPOSE 8000

# Set user to www-data
USER solar-grabber

# Set container home directory
WORKDIR /app

# Run Solar Grabber
ENTRYPOINT [ "/app/solar-grabber" ]
