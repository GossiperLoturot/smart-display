# smart-display

This application displays information like date and time.

## Usages

1. Clone this repository.
```sh
git clone https://github.com/GossiperLoturot/smart-display
```

2. Launch the API server.
```sh
cargo run --release --state-filepath=./state.json --address=localhost:50822
```

3. Build and serve the web page.
```sh
cd ./site
bun i
VITE_API_URL=http:/localhost:50822 bun run build
bun run preview
```

### API server command options

|Command Options|Description|Default|
|--|--|--|
|state-filepath|persistent data file path||
|address|The address and port on which the API server is listening||
|image-width|Image width when saving image to buffer|800|
|image-height|Image height when saving image to buffer|480|
|th-combine|Whether to read temperature and humidity||
|th-combine-filepath|Temperature and humidity file path||
|th-combine-duration-secs|A millisecond interval to read temperature and humidity file||

#### Temperature and Humidity File Content

```json
{
    "temperature": 25.0,
    "humidity": 40.5
}
```

### Web Page Environment Variables

|Environment Variables|Description|Default|
|--|--|--|
|VITE_WIDTH|Expected width of the display|800|
|VITE_HEIGHT|Expected height of the display|480|
|VITE_API_URL|Expected API server address and port|localhost:50822|
|VITE_POLLING_INTERVAL|A millisecond interval to communicate with the API server|250|
