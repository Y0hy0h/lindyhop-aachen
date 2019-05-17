# Lindy Hop Aachen

[![Build Status](https://travis-ci.org/Y0hy0h/lindyhop-aachen.svg?branch=master)](https://travis-ci.org/Y0hy0h/lindyhop-aachen)

A website about all things Lindy Hop in Aachen.

## Development
Set up the database by downloading the [Diesel CLI], creating a directory `./db`, and executing `diesel setup --database-url db/db.sqlite`.

Using [cargo-watch], you can recompile Rust on file changes. Install it using `cargo install cargo-watch`. Also install the [Node.js] dependencies with [Yarn] by running `yarn install`.

Compiling server, styles, and admin, and recompiling each of them on changes, is done with
```bash
yarn watch
```

[cargo-watch]: https://github.com/passcod/cargo-watch
[Node.js]: https://nodejs.org/en/
[Yarn]: https://yarnpkg.com/lang/en/
[Diesel CLI]: https://github.com/diesel-rs/diesel/tree/master/diesel_cli#installation