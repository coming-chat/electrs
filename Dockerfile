FROM rust:1.56.1 as builder

RUN apt update \
    && apt install -y libclang-dev

WORKDIR /code
COPY ./ ./
RUN cargo build --release

FROM alpine:latest
WORKDIR /code
COPY --from=builder /code/target/release/electrs /usr/local/bin
COPY ./run .

RUN chmod +x ./run

ENTRYPOINT ["./run"]