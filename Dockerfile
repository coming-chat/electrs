FROM rust:1.56.1 as builder

WORKDIR /code
COPY ./ ./
RUN cargo build --release

FROM alpine:latest
WORKDIR /code
COPY --from=builder /code/target/release/ /usr/local/bin
COPY ./run .

RUN chmod +x ./run

ENTRYPOINT ["./run"]