# TODO: go back to alpine once libmp3lame dependency is fixed
FROM rust:1.69-slim-buster AS build-rust

RUN apt update
RUN apt install -y libmp3lame-dev

RUN mkdir -p /workspace/src
WORKDIR /workspace

ADD Cargo.toml Cargo.lock /workspace
ADD src /workspace/src

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN --mount=type=cache,target=/workspace/target cargo build --release && cp target/release/edifm .


FROM ruby:2.7.8-slim-buster AS build-ruby

RUN apt update
RUN apt install -y build-essential libsqlite3-dev

RUN mkdir -p /srv/web
COPY ./web /srv/web
WORKDIR /srv/web

RUN bundle config set --local path vendor/bundle
RUN bundle install


FROM ruby:2.7.8-slim-buster AS deploy

RUN mkdir -p /data/catalog
RUN ln -nsf /data/catalog /srv/catalog

RUN apt update
RUN apt install -y libmp3lame0 bash sqlite3 icecast2 nginx s6

RUN mkdir -p /srv/web
COPY --from=build-ruby /srv/web /srv/web

RUN mkdir -p /usr/local/bundle
COPY --from=build-ruby /usr/local/bundle /usr/local/bundle

RUN mkdir -p /srv
COPY --from=build-rust /workspace/edifm /srv/edifm

RUN mkdir -p /srv/migrations
COPY migrations /srv/migrations

RUN mkdir -p /srv/script
COPY script /srv/script

ADD deploy/icecast.xml /etc/icecast2/icecast.xml
ADD deploy/nginx.conf /etc/nginx/nginx.conf
ADD deploy/log-service /usr/local/bin/log-service

RUN mkdir -p /srv/services
ADD deploy/services /srv/services

# ENTRYPOINT /bin/bash
CMD ["/usr/bin/s6-svscan", "/srv/services"]
