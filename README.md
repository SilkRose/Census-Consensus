# April Fools 2026

Hi .3c

## Dependencies setup

You need:

- a rust compiler toolchain installed via [rustup](https://rustup.rs)
  - Once installed, running any rust command in this directory will cause rustc to automatically install (if needed) and run the right version of the tool. You will know if rustup is installing something.
- [pnpm](https://pnpm.io)
- [cargo-leptos](https://crates.io/crates/cargo-leptos)
  - To install: `cargo install --locked cargo-leptos`
- a [postgres](https://www.postgresql.org) instance set up and available

## Environment setup

This project has a dotenv implementation configured, so you may use a `.env` file in the root of the project directory to configure environment variables.

Environment variables:

- `POSTGRES_URL`: postgres connection URL, in the format of `postgres://<username>:<password>@<hostname>:<port>/<databasename>`
  - ex. `postgres://postgres:root@localhost:5432/aprilfools`, with username "postgres", password "root", hostname "localhost", port "5432", database name "aprilfools"

## Running development server

Run the command `cargo leptos watch`, and a development server should start at `localhost:3000` with a live reload server.

## Building

Run `cargo leptos build` for a debug build, or `cargo leptos build -r` for a release build.

## Deploying

Move the following files to a directory on your deployment server:

- the server binary at `target/release/april-fools-2026` to `/april-fools-2026`
- the site dir at `target/site` to `/site`

Add the following environment variables:

- `POSTGRES_URL`: postgres connection URL (see [above](#environment-setup))

Then, run the server binary inside of this dir (`./april-fools-2026`).
