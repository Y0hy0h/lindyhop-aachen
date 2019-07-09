FROM node:lts AS node
WORKDIR /node
# Cache compiled dependencies (inspired by http://whitfin.io/speeding-up-rust-docker-builds/)
COPY ./package.json ./package.json
RUN npm install
COPY ./admin/elm.json ./admin/elm.json
RUN mkdir ./admin/src && echo "import Html\nmain = Html.text \"Hello World\"" >> ./admin/src/Main.elm
RUN npm run compile:admin
RUN rm -r ./admin/src
# Actual build
COPY ./styles ./styles
COPY ./admin/src ./admin/src
RUN npm run build:node

FROM ekidd/rust-musl-builder:nightly-2019-06-08 AS rust
# Cache compiled dependencies (see http://whitfin.io/speeding-up-rust-docker-builds/)
RUN USER=root cargo init --bin
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release --target x86_64-unknown-linux-musl
# Ensure Cargo rebuilds. Leaving build files might make Cargo skip rebuilding. (See end of section http://whitfin.io/speeding-up-rust-docker-builds/#optimizingbuildtimes)
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/lindyhop_aachen*
RUN rm -r ./src
# Actual build
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./Rocket.toml ./Rocket.toml
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /lindyhop-aachen
RUN mkdir ./db/
COPY --from=node /node/static ./static
COPY --from=node /node/admin/dist ./admin/dist
COPY --from=rust /home/rust/src/target/x86_64-unknown-linux-musl/release/lindyhop-aachen ./lindyhop-aachen
COPY --from=rust /home/rust/src/Rocket.toml ./Rocket.toml
CMD [ "./lindyhop-aachen" ]