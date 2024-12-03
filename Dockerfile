# syntax=docker/dockerfile:1
FROM rust:alpine
COPY Cargo.* .
COPY src ./src
COPY tferris_txts/ ./data
RUN apk add musl-dev # https://stackoverflow.com/questions/6329887/how-to-fix-linker-error-cannot-find-crt1-o
RUN cargo build --release --bin web
EXPOSE 7878
CMD ["target/release/web", "data"]
#CMD ["sh"]
