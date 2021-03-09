###############
# Build
###############
FROM rust:1.50.0 AS build

# update crates.io index for build cache
RUN cargo search tokio

WORKDIR /app

COPY tools tools
COPY Cargo.* .
COPY src src
COPY simulated-annealing simulated-annealing

RUN cargo build --release --bin=lambda

###############
# Run
###############
FROM public.ecr.aws/lambda/provided:al2

COPY tools/in2 /in2
COPY --from=build /app/target/release/lambda ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["dummy.name"]
