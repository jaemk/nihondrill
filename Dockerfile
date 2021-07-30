FROM rust:1.52 as builder

RUN cargo install migrant --features postgres

# create a new empty shell
RUN USER=root cargo new --bin nihondrill
WORKDIR /nihondrill

# copy over your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release
RUN rm src/*.rs

# copy all source/static/resource files
COPY ./src ./src
COPY ./sqlx-data.json ./sqlx-data.json

# build for release
RUN rm ./target/release/deps/nihondrill*

ENV SQLX_OFFLINE=true
RUN cargo build --release

# copy over git dir and embed latest commit hash
COPY ./.git ./.git
# make sure there's no trailing newline
RUN git rev-parse HEAD | awk '{ printf "%s", $0 >"commit_hash.txt" }'
RUN rm -rf ./.git

# package
FROM debian:buster-slim
RUN mkdir /nihondrill
WORKDIR /nihondrill

COPY ./bin ./bin
COPY --from=builder /nihondrill/target/release/nihondrill ./bin/nihondrill
COPY --from=builder /nihondrill/commit_hash.txt ./commit_hash.txt

# copy all static files
COPY ./Migrant.toml ./Migrant.toml
COPY ./migrations ./migrations
#COPY ./templates ./templates
#COPY ./assets ./assets

CMD ["./bin/start.sh"]

