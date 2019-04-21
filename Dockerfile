FROM node:lts AS node
WORKDIR /node
COPY ./package.json .
RUN npm install
COPY ./admin/elm.json ./admin
# Cache compiled dependencies (inspired by http://whitfin.io/speeding-up-rust-docker-builds/)
RUN mkdir src && echo "import Html\nmain = Html.text \"Hello World\"" >> src/Main.elm
RUN npm run compile:admin
RUN rm -r src
# Actual build
COPY ./styles ./styles
COPY ./admin/src ./admin/src
RUN npm run build

FROM rust:latest AS rust
RUN rustup toolchain install nightly-2019-03-23 && rustup default nightly-2019-03-23
# Cache compiled dependencies (see http://whitfin.io/speeding-up-rust-docker-builds/)
WORKDIR /
RUN USER=root cargo new lindyhop-aachen --bin
WORKDIR /lindyhop-aachen
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release
RUN rm ./target/release/deps/lindyhop_aachen*
RUN rm -r ./src
# Actual build
COPY ./src ./src
RUN cargo build --release

FROM rust:slim
WORKDIR /lindyhop-aachen
COPY --from=node /node/static ./static
COPY --from=node /node/admin/dist ./admin/dist
COPY --from=rust /lindyhop-aachen/target/release/lindyhop-aachen .
CMD [ "./lindyhop-aachen" ]