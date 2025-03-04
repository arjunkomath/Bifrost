# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------
FROM rust:1.85 as build

WORKDIR /usr/src/bifrost
COPY . .

RUN cargo install --path .

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl-dev \
    libsqlite3-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the architecture argument (arm64, i.e. aarch64 as default)
# For amd64, i.e. x86_64, you can append a flag when invoking the build `... --build-arg "ARCH=x86_64"`
# ARG ARCH=aarch64

# Application files
COPY --from=build /usr/local/cargo/bin/bifrost /usr/local/bin/bifrost

# Copy the templates folder into the container
COPY templates /templates

EXPOSE 8080

CMD ["bifrost"]
