# smart-display

![display](https://raw.githubusercontent.com/GossiperLoturot/smart-display/main/images/display.webp)
![menu](https://raw.githubusercontent.com/GossiperLoturot/smart-display/main/images/menu.webp)

*Picture from [Unsplash.com](https://unsplash.com/)*

This application displays information like date and time.

## Usages

1. Clone this repository.
```sh
git clone https://github.com/GossiperLoturot/smart-display
```

2. Build the web page.
```sh
cd ./site
bun install
bun run build

# With environment variables
VITE_POLLING_INTERVAL=250 bun run build
```

3. Launch the server.
```sh
cargo run --release

# With extra data file
cargo run --release -- --extra \
    --extra-filepath /path/to/extra.json \
    --extra-duration-secs 60
```

### Server Command Options

|Command Options|Description|Default|
|--|--|--|
|state-filepath|persistent data file path|./state.json|
|address|The address and port that the server is listening on|0.0.0.0:50822|
|html|The web page served by the server|./html|
|image-width|Image width when saving image to buffer|780|
|image-height|Image height when saving image to buffer|460|
|extra|Whether to read extra data file||
|extra-filepath|extra data file path||
|extra-duration-secs|A millisecond interval to read extra data file||

#### Structure of Extra Data

Currently, extra data only contains temperature and humidity.

```json
{
    "temperature": 25.0,
    "humidity": 40.5
}
```

### Web Page Environment Variables

|Environment Variables|Description|Default|
|--|--|--|
|VITE_WIDTH|Expected width of the display|780|
|VITE_HEIGHT|Expected height of the display|460|
|VITE_API_URL|Expected API server address and port|window.location.host|
|VITE_POLLING_INTERVAL|A millisecond interval to communicate with the API server|250|
