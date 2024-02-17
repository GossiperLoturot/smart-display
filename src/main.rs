use clap::Parser;
use warp::Filter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = args::Args::parse();
    let args = std::sync::Arc::new(args);

    let state = match server::State::read(&args.state_filepath).await {
        Ok(state) => {
            tracing::info!("load state file {:?}", args.state_filepath);
            tracing::info!("use loaded state file");
            state
        }
        Err(err) => {
            tracing::warn!("failed to load state file {:?}", err);
            tracing::info!("use default state file");
            server::State::default()
        }
    };
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let buffer = picture_buffer::Buffer::default();
    let buffer = std::sync::Arc::new(tokio::sync::Mutex::new(buffer));

    let filter = warp::path("api")
        .and(
            polling::handle(state.clone())
                .or(picture_index::handle(state.clone()))
                .or(picture_apply::handle(args.clone(), state.clone()))
                .or(picture_create::handle(args.clone(), state.clone()))
                .or(picture_delete::handle(args.clone(), state.clone()))
                .or(picture_buffer::handle(buffer))
                .recover(reject::handle),
        )
        .or(warp::fs::dir(args.dist_dirpath.clone()))
        .or(warp::fs::file(args.dist_filepath.clone()));

    tracing::info!("start background state routine");
    tokio::spawn(server::run_picture_shuffle(state.clone()));
    tokio::spawn(server::run_sensor_fetch(args.clone(), state));

    tracing::info!("start listening {:?}", args.address);
    warp::serve(filter).run(args.address).await;
}

mod args {
    pub type SyncArgs = std::sync::Arc<Args>;

    #[derive(clap::Parser)]
    #[clap(
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        author = env!("CARGO_PKG_AUTHORS"),
        about = env!("CARGO_PKG_DESCRIPTION"),
    )]
    pub struct Args {
        #[arg(long)]
        pub state_filepath: std::path::PathBuf,
        #[arg(long)]
        pub dist_dirpath: std::path::PathBuf,
        #[arg(long)]
        pub dist_filepath: std::path::PathBuf,
        #[arg(long)]
        pub address: std::net::SocketAddr,
        #[arg(long)]
        pub sensor: bool,
        #[arg(long, required_if_eq("sensor", "true"))]
        pub sensor_filepath: Option<std::path::PathBuf>,
        #[arg(long, required_if_eq("sensor", "true"))]
        pub sensor_duration: Option<f32>,
    }
}

mod server {
    use anyhow::Context;
    use rand::{seq::IteratorRandom, SeedableRng};

    use crate::args;

    pub type SyncState = std::sync::Arc<tokio::sync::Mutex<State>>;

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub duration_secs: f32,
        pub url_set: std::collections::HashSet<String>,

        #[serde(skip)]
        pub url: Option<String>,
        #[serde(skip)]
        pub temperature: Option<f32>,
        #[serde(skip)]
        pub humidity: Option<f32>,
    }

    impl State {
        pub async fn read(filepath: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
            let buf = tokio::fs::read(filepath)
                .await
                .context("failed to read state from file")?;
            let slf =
                serde_json::from_slice(&buf).context("failed to deserialize state from json")?;
            Ok(slf)
        }

        pub async fn write(&self, filepath: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
            let buf = serde_json::to_vec(self).context("failed to serialize state to json")?;
            tokio::fs::write(filepath, buf)
                .await
                .context("failed to write state to file")?;
            Ok(())
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                duration_secs: 60.0,
                url_set: std::collections::HashSet::new(),

                url: None,
                temperature: None,
                humidity: None,
            }
        }
    }

    pub async fn run_picture_shuffle(state: SyncState) {
        let mut rng = rand::rngs::StdRng::from_entropy();

        loop {
            let mut state = state.lock().await;

            match state.url_set.iter().choose(&mut rng) {
                Some(url) => {
                    tracing::debug!("success to set picture url {:?}", url);
                    state.url = Some(url.clone());
                }
                None => {
                    tracing::debug!("failed to set picture url");
                }
            }

            let duration_secs = state.duration_secs;

            drop(state);

            tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct SensorContent {
        temperature: f32,
        humidity: f32,
    }

    pub async fn run_sensor_fetch(args: args::SyncArgs, state: SyncState) -> anyhow::Result<()> {
        let filepath = args
            .sensor_filepath
            .as_ref()
            .context("no parse sensor filepath")?;
        let duration_secs = args.sensor_duration.context("no parse sensor duration")?;

        loop {
            let file = std::fs::File::open(filepath)?;
            let content = serde_json::from_reader::<_, SensorContent>(file)?;

            tracing::debug!("read sensor content {:?}", content);

            let mut state = state.lock().await;

            state.temperature = Some(content.temperature);
            state.humidity = Some(content.humidity);

            drop(state);

            tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
        }
    }
}

mod picture_buffer {
    use anyhow::Context;
    use warp::Filter;

    use crate::reject;

    type SyncBuffer = std::sync::Arc<tokio::sync::Mutex<Buffer>>;

    struct ImageEntry {
        bytes: Vec<u8>,
        rgb: [u8; 3],
    }

    pub struct Buffer {
        client: reqwest::Client,
        images: std::collections::HashMap<String, ImageEntry>,
    }

    impl Default for Buffer {
        fn default() -> Self {
            Self {
                client: reqwest::Client::new(),
                images: std::collections::HashMap::new(),
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        buffer: SyncBuffer,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            buffer: SyncBuffer,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit picture buffer {:?}", request);

            let mut buffer = buffer.lock().await;

            let image = match buffer.images.get(&request.url) {
                Some(image) => {
                    tracing::debug!("use cache image {:?}", request.url);
                    image
                }
                None => {
                    tracing::info!("buffer image {:?}", request.url);
                    let image = buffering(&buffer.client, &request.url)
                        .await
                        .map_err(reject::to)?;

                    tracing::debug!("create cache image {:?}", request.url);
                    buffer.images.entry(request.url).or_insert(image)
                }
            };

            let response = warp::http::Response::builder()
                .status(200)
                .header("Content-Type", "image/jpeg")
                .header("X-Repr-R", image.rgb[0].to_string())
                .header("X-Repr-G", image.rgb[1].to_string())
                .header("X-Repr-B", image.rgb[2].to_string())
                .body(image.bytes.clone())
                .context("failed to create http response for reply")
                .map_err(reject::to)?;

            Ok(response)
        }

        warp::path("buffer")
            .and(warp::get())
            .and(warp::query::<Request>())
            .map(move |request| (request, buffer.clone()))
            .untuple_one()
            .and_then(handle)
    }

    async fn buffering(client: &reqwest::Client, url: &str) -> anyhow::Result<ImageEntry> {
        let response = client
            .get(url)
            .send()
            .await
            .context("failed to get http response")?;

        let is_image = response
            .headers()
            .get("Content-Type")
            .map(|value| value.as_bytes().starts_with(b"image/"))
            .unwrap_or_default();

        anyhow::ensure!(is_image, "http content type must be image/*");

        let bytes = response
            .bytes()
            .await
            .context("failed to get http response body")?;
        let image = image::load_from_memory(&bytes)
            .context("failed to load image from http response body")?;

        let mut buf = std::io::Cursor::new(vec![]);
        image
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .context("failed to encode into jpeg from image")?;
        let bytes = buf.into_inner();

        let rgb = compute_repr_rgb(image).await.0;

        Ok(ImageEntry { bytes, rgb })
    }

    async fn compute_repr_rgb(image: image::DynamicImage) -> image::Rgb<u8> {
        let pixels = image.to_rgb8();
        let pixels_len = pixels.len() / 3;

        let r_iter = pixels.iter().step_by(3);
        let g_iter = pixels.iter().skip(1).step_by(3);
        let b_iter = pixels.iter().skip(2).step_by(3);

        let r = (r_iter.fold(0, |acc, r| acc + *r as usize) / pixels_len) as u8;
        let g = (g_iter.fold(0, |acc, g| acc + *g as usize) / pixels_len) as u8;
        let b = (b_iter.fold(0, |acc, b| acc + *b as usize) / pixels_len) as u8;

        image::Rgb([r, g, b])
    }
}

mod polling {
    use futures::{SinkExt, StreamExt};
    use warp::Filter;

    use crate::server;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        date_time: chrono::DateTime<chrono::Local>,
        url: Option<String>,
        temperature: Option<f32>,
        humidity: Option<f32>,
    }

    pub fn handle(
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn ws_handle(ws: warp::ws::WebSocket, state: server::SyncState) {
            tracing::info!("established websocket connection");

            let (mut tx, mut rx) = ws.split();

            while rx.next().await.is_some() {
                let state = state.lock().await;

                let response = Response {
                    date_time: chrono::Local::now(),
                    url: state.url.clone(),
                    temperature: state.temperature,
                    humidity: state.humidity,
                };

                match serde_json::to_string(&response) {
                    Ok(text) => {
                        let message = warp::ws::Message::text(text);
                        match tx.send(message).await {
                            Ok(_) => {
                                tracing::debug!("success to send message");
                            }
                            Err(err) => {
                                tracing::warn!("failed to send message {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("failed to serialize {:?}", err);
                    }
                }
            }
        }

        warp::path("polling")
            .and(warp::ws())
            .map(move |ws| (ws, state.clone()))
            .untuple_one()
            .map(|ws: warp::ws::Ws, state| ws.on_upgrade(|ws| ws_handle(ws, state)))
    }
}

mod picture_index {
    use warp::Filter;

    use crate::server;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        duration_secs: f32,
        urls: Vec<String>,
        url: Option<String>,
    }

    pub fn handle(
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(state: server::SyncState) -> impl warp::Reply {
            tracing::info!("hit picture index");

            let state = state.lock().await;

            let response = Response {
                duration_secs: state.duration_secs,
                urls: state.url_set.iter().cloned().collect::<Vec<_>>(),
                url: state.url.clone(),
            };

            warp::reply::json(&response)
        }

        warp::path("config")
            .and(warp::get())
            .map(move || state.clone())
            .then(handle)
    }
}

mod picture_apply {
    use warp::Filter;

    use crate::{args, reject, server};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        duration_secs: Option<f32>,
        url: Option<String>,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: server::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit picture apply {:?}", request);

            let mut state = state.lock().await;

            if let Some(url) = request.url {
                tracing::info!("set current url {:?}", url);
                state.url = Some(url);
            }

            if let Some(duration_secs) = request.duration_secs {
                tracing::info!("set duration secs {:?}", duration_secs);
                state.duration_secs = duration_secs;
            }

            state
                .write(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::patch())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod picture_create {
    use warp::Filter;

    use crate::{args, reject, server};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: server::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit picture create {:?}", request);

            let mut state = state.lock().await;

            tracing::info!("insert url {:?}", request.url);
            state.url_set.insert(request.url);

            state
                .write(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod picture_delete {
    use warp::Filter;

    use crate::{args, reject, server};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: server::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit picture delete {:?}", request);

            let mut state = state.lock().await;

            tracing::info!("remove url {:?}", request.url);
            state.url_set.remove(&request.url);

            state
                .write(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::delete())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod reject {
    #[derive(Debug)]
    pub struct Rejection(anyhow::Error);

    pub fn to(value: anyhow::Error) -> Rejection {
        Rejection(value)
    }

    impl warp::reject::Reject for Rejection {}

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        status_code: u16,
        message: String,
    }

    pub async fn handle(
        err: warp::Rejection,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let status_code;
        let message;

        if err.is_not_found() {
            status_code = warp::http::StatusCode::NOT_FOUND;
            message = "NOT_FOUND".to_string();
        } else if let Some(err) = err.find::<Rejection>() {
            status_code = warp::http::StatusCode::BAD_REQUEST;
            message = err.0.to_string();
        } else {
            status_code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
            message = "UNHANDLED_REJECTION".to_string();
        }

        let response = Response {
            status_code: status_code.as_u16(),
            message,
        };

        Ok(warp::reply::json(&response))
    }
}
