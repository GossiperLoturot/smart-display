![GitHub](https://img.shields.io/github/license/GossiperLoturot/smart-display)

# smart-display

This application shows information such as date and time.

## Usages

1. Clone this repository by `git clone https://github.com/GossiperLoturot/smart-display`.
2. Run `npm i` and `npm run build` to generate static sites.
3. Run `cargo run --release` at `/api` for launch server.

|Environment Variants|Description|Default|
|--|--|--|
|DIST_DIR|A place that static sites exists|dist|
|DIST_FILE|A place that index.html exists|DIST_DIR/index.html|
|ADDR|A address that web server hosts|127.0.0.1:3000|

## Developments

1. Run `cargo run` at `/api` to launch server for developments.
2. Run `npm run dev` to launch client.

## Productions

1. Extracts binary and static sites from `/api/target/release/smart-display` and `/dist` after build.
2. Place there at new directory and run `ADDR=0.0.0.0:3000 ./smart-display` to launch server.
