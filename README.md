# April Fools 2026

Hi .3c

## Dependencies setup

You need:

- a rust compiler toolchain installed via [rustup](https://rustup.rs)
  - Installation: <https://rustup.rs>
  - Once installed, running any rust command in this directory will cause rustc to automatically install (if needed) and run the right version of the tool (it will tell you when it is installing something).
- [pnpm](https://pnpm.io)
  - Installation: <https://pnpm.io/installation>
  - Run `pnpm install` to install the necessary npm dependencies in the workspace
- a [postgres](https://www.postgresql.org) instance set up and available
  - Installation: <https://www.postgresql.org/download/>
  - A container image is also available, if you would prefer: <https://hub.docker.com/_/postgres>
- [sqlx-cli](https://crates.io/crates/sqlx-cli)
  - Installation: `cargo install --locked sqlx-cli`
  - You only need the `postgres` and a TLS library feature to use sqlx for this project, so you may optionally add `--no-default-features --features postgres,rustls` to the install command
  - If you wish to limit the amount of binaries installed, you may choose to only install either the `cargo-sqlx` or `sqlx` commands with `--bin cargo-sqlx` or `--bin sqlx`, respectively. You'd then invoke the CLI with `cargo sqlx` or `sqlx`
- [tailwindcss](https://tailwindcss.com) CLI
  - Installation: `pnpm install --global @tailwindcss/cli`
  - The CLI depends on the `tailwindcss` package being installed in the workspace (which it is!)
- [Trunk](https://trunkrs.dev)
  - Installation: `cargo install trunk --no-default-features`
- [wasm-bindgen CLI](https://crates.io/crates/wasm-bindgen)
  - Installation: `cargo install wasm-bindgen-cli --version 0.2.108` (must be exactly version 0.2.108 for this project)
- [wasm-opt](https://crates.io/crates/wasm-opt)
  - Installation: `cargo install wasm-opt`

## Project setup

Before compiling the project for the first time, you will need to use the sqlx cli to run migrations on the database, so that sqlx can typecheck and infer types during the compilation process. The command is `cargo sqlx migrate run`. See below for information on setting the required `DATABASE_URL` environment variable for connecting to the database.

For environment variables, you may use a `.env` file in the root of the project directory to configure them.

### Environment variables

- `DATABASE_URL`: postgres connection URL, in the format of `postgres://<username>:<password>@<hostname>:<port>/<databasename>`
  - ex. `postgres://postgres:root@localhost:5432/aprilfools`, with username "postgres", password "root", hostname "localhost", port "5432", database name "aprilfools"

## Running development server (currently outdated) <!-- todo fix this -->

Run the command `cargo leptos watch`, and a development server should start at `localhost:3000` with live reload.

## Building (currently outdated) <!-- todo fix this -->

Run `cargo leptos build` for a debug build, or `cargo leptos build -r` for a release build. Note: a postgres instance with migrations ran is needed for any form of checking or compilation (see [#project-setup](#project-setup) for more information).

## Deploying (currently outdated) <!-- todo fix this -->

Build the project as describe above, then move the following files to a directory on your deployment server:

- the server binary at `target/release/april-fools-2026` to `/april-fools-2026`
- the site dir at `target/site` to `/site`

Set the following environment variables:

- `DATABASE_URL`: postgres connection URL (see [above](#environment-variables))
- `LEPTOS_SITE_ADDR`: address to bind the server to, if you need to override the default of `127.0.0.1:3000`

Then, run the server binary inside of this dir (`./april-fools-2026`).
