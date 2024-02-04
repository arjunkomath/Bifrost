FROM rust:latest as build

WORKDIR /usr/src/bifrost
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/local/cargo/bin/bifrost /usr/local/bin/bifrost

EXPOSE 8080

CMD ["bifrost"]
