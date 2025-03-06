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
FROM alpine:latest

# Install required dependencies
RUN apk add --no-cache \
    openssl \
    sqlite-dev \
    ca-certificates \
    libc6-compat

COPY --from=build /usr/local/cargo/bin/bifrost /usr/local/bin/bifrost

EXPOSE 8080

CMD ["bifrost"]
