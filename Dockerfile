FROM public.ecr.aws/docker/library/rust:latest AS build

WORKDIR /home/code
ADD ./migrations ./migrations
ADD ./src/ ./src/
ADD ./templates ./templates/
ADD Cargo.toml .
ADD rust-toolchain.toml .

RUN cargo build --release

FROM public.ecr.aws/docker/library/rust:latest

RUN apt-get install -y curl
HEALTHCHECK --interval=30s --timeout=3s CMD curl -X HEAD -f http://localhost:8102/ || exit 1

WORKDIR /srv

COPY --from=build /home/code/target/release/you_up /srv/you_up
ADD ./static ./static/

CMD ["/srv/you_up"]
