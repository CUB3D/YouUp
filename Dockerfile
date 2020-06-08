FROM rust:latest

WORKDIR /home/code

ADD ./migrations ./migrations
ADD ./src/ ./src/
ADD ./static ./static/
ADD ./templates ./templates/
ADD Cargo.toml .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
