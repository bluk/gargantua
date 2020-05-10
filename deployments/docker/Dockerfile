# Copyright 2020 Bryant Luk
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

FROM rust:slim as build

RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release; \
    rm -rf src

COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim

RUN mkdir app
WORKDIR /app

COPY --from=build /app/target/release/gargantua .

EXPOSE 8080

ARG SOURCE_COMMIT=unknown
ENV SOURCE_COMMIT $SOURCE_COMMIT

ARG IMAGE_TAG=unknown
ENV IMAGE_TAG $IMAGE_TAG

ENTRYPOINT ["/app/gargantua"]
