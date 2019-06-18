# Lindy Hop Aachen

[![Build Status](https://travis-ci.org/Y0hy0h/lindyhop-aachen.svg?branch=master)](https://travis-ci.org/Y0hy0h/lindyhop-aachen)

A website about all things Lindy Hop in Aachen.

## Development
Set up the database by downloading the [Diesel CLI] with `cargo install diesel_cli --no-default-features --features "sqlite-bundled"`, creating a directory `./db`, and executing `diesel setup --database-url db/db.sqlite`.

Using [cargo-watch], you can recompile Rust on file changes. Install it using `cargo install cargo-watch`. Also install the [Node.js] dependencies with [Yarn] by running `yarn install`.

Compiling server, styles, and admin, and recompiling each of them on changes, is done with
```bash
yarn watch
```

The Dockerfile can be used to compile everything into a distributable form. The artifacts will be in `/lindyhop-aachen`, inside of which is the executable you need to run called `lindyhop-aachen`.

1. `docker build -t lindy .`
2. `docker create --name lindy lindy`.
3. `docker cp lindy:/lindyhop-aachen <your_output_dir>`
3. Execute `<your_output_dir>/lindyhop-aachen`

## Deployment
You can download a precompiled binary along with all necessary files from this [repository's releases](./releases).

In case you want to keep your old database, be aware that you might need to migrate it to a new format.

[cargo-watch]: https://github.com/passcod/cargo-watch
[Node.js]: https://nodejs.org/en/
[Yarn]: https://yarnpkg.com/lang/en/
[Diesel CLI]: https://github.com/diesel-rs/diesel/tree/master/diesel_cli#installation