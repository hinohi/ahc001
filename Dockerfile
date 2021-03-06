###############
# Build
###############
FROM rust:1.50.0 AS build

WORKDIR /app

COPY Cargo.* .
COPY src src

RUN cargo build --release --bin=a

###############
# Run
###############
FROM gcr.io/distroless/cc

COPY --from=build /app/target/release/a /

CMD ["/a"]
