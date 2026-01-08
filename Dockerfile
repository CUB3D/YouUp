FROM public.ecr.aws/docker/library/rust:latest

WORKDIR /home/code

ADD ./migrations ./migrations
ADD ./src/ ./src/
ADD ./static ./static/
ADD ./templates ./templates/
ADD Cargo.toml .
ADD rust-toolchain.toml .

RUN cargo build --release

HEALTHCHECK --interval=30s --timeout=3s CMD curl -X HEAD -f http://localhost:8102/ || exit 1

CMD ["cargo", "run", "--release"]
