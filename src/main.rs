use clap::Parser;
use warp::Filter;

mod args {
    pub type SyncArgs = std::sync::Arc<Args>;

    #[derive(Debug, Clone, clap::Parser)]
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
        pub address: std::net::SocketAddr,
        #[arg(long, default_value_t = 800)]
        pub image_width: u32,
        #[arg(long, default_value_t = 480)]
        pub image_height: u32,
        #[arg(long)]
        pub th_combine: bool,
        #[arg(long, required_if_eq("th_combine", "true"))]
        pub th_combine_filepath: Option<std::path::PathBuf>,
        #[arg(long, required_if_eq("th_combine", "true"))]
        pub th_combine_duration_secs: Option<f32>,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = args::Args::parse();
    let args = std::sync::Arc::new(args);

    let state = match state::State::read_from_file(&args.state_filepath).await {
        Ok(state) => {
            tracing::info!("load state file {:?}", args.state_filepath);
            tracing::info!("use loaded state file");
            state
        }
        Err(err) => {
            tracing::warn!("failed to load state file {:?}", err);
            tracing::info!("use default state file");
            state::State::new()
        }
    };
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let image_buffer = image_buffer::ImageBuffer::new();
    let image_buffer = std::sync::Arc::new(tokio::sync::Mutex::new(image_buffer));

    let filter = polling::handle(state.clone())
        .or(image_index::handle(state.clone()))
        .or(image_modify::handle(args.clone(), state.clone()))
        .or(image_create::handle(args.clone(), state.clone()))
        .or(image_delete::handle(args.clone(), state.clone()))
        .or(image_buffer::handle(args.clone(), image_buffer.clone()))
        .recover(reject::handle)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(["GET", "POST"])
                .allow_headers(["Content-Type"])
                .build(),
        );

    tracing::info!("start background state routine");
    tokio::spawn(state::image_shuffling_loop(state.clone()));
    tokio::spawn(state::th_combine_fething_loop(args.clone(), state));

    tracing::info!("start listening {:?}", args.address);
    warp::serve(filter).run(args.address).await;
}

mod state {
    use crate::args;
    use anyhow::Context;

    pub type SyncState = std::sync::Arc<tokio::sync::Mutex<State>>;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub duration_secs: f32,
        pub image_urls: std::collections::HashSet<String>,
        #[serde(skip)]
        pub image_url: Option<String>,
        #[serde(skip)]
        pub th_combine: Option<ThCombine>,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                duration_secs: 60.0,
                image_urls: Default::default(),
                image_url: Default::default(),
                th_combine: Default::default(),
            }
        }

        pub async fn read_from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
            let buf = tokio::fs::read(path)
                .await
                .context("failed to read state from file")?;
            let slf =
                serde_json::from_slice(&buf).context("failed to deserialize state from json")?;
            Ok(slf)
        }

        pub async fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
            let buf = serde_json::to_vec(self).context("failed to serialize state to json")?;
            tokio::fs::write(path, buf)
                .await
                .context("failed to write state to file")?;
            Ok(())
        }
    }

    pub async fn image_shuffling_loop(state: SyncState) {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::from_entropy();

        loop {
            let mut state = state.lock().await;

            match rand::seq::IteratorRandom::choose(state.image_urls.iter(), &mut rng) {
                Some(image_url) => {
                    tracing::debug!("success to set image url {:?}", image_url);
                    state.image_url = Some(image_url.clone());
                }
                None => {
                    tracing::debug!("failed to set image url");
                }
            }

            let duration_secs = std::time::Duration::from_secs_f32(state.duration_secs);

            drop(state);

            tokio::time::sleep(duration_secs).await;
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ThCombine {
        pub temperature: f32,
        pub humidity: f32,
    }

    pub async fn th_combine_fething_loop(args: args::SyncArgs, state: SyncState) {
        let path = match args.th_combine_filepath.as_ref() {
            Some(path) => {
                tracing::info!("parse th combine file path {:?}", path);
                path
            }
            None => {
                tracing::info!("no th combine file path");
                return;
            }
        };

        let secs = match args.th_combine_duration_secs {
            Some(secs) => {
                tracing::info!("parse th combine duration {:?}", secs);
                std::time::Duration::from_secs_f32(secs)
            }
            None => {
                tracing::info!("no th combine duration");
                return;
            }
        };

        loop {
            let file = match std::fs::File::open(path) {
                Ok(file) => {
                    tracing::debug!("success to read th combine file {:?}", path);
                    file
                }
                Err(err) => {
                    tracing::warn!("failed to read th combine file {:?}", err);
                    tokio::time::sleep(secs).await;
                    continue;
                }
            };

            let th_combine = match serde_json::from_reader::<_, ThCombine>(file) {
                Ok(th_combine) => {
                    tracing::debug!("success to parse th combine {:?}", th_combine);
                    th_combine
                }
                Err(err) => {
                    tracing::warn!("failed to parse th combine {:?}", err);
                    tokio::time::sleep(secs).await;
                    continue;
                }
            };

            let mut state = state.lock().await;

            state.th_combine = Some(th_combine);

            drop(state);

            tokio::time::sleep(secs).await;
        }
    }
}

mod polling {
    use crate::state;
    use warp::Filter;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        date_time: chrono::DateTime<chrono::Local>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        th_combine: Option<state::ThCombine>,
    }

    pub fn handle(
        state: state::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(state: state::SyncState) -> impl warp::Reply {
            // silent
            // tracing::info!("hit polling api");

            let state = state.lock().await;

            let response = Response {
                date_time: chrono::Local::now(),
                image_url: state.image_url.clone(),
                th_combine: state.th_combine.clone(),
            };

            warp::reply::json(&response)
        }

        warp::path("polling")
            .and(warp::get())
            .map(move || state.clone())
            .then(handle)
    }
}

mod image_buffer {
    use crate::{args, reject};
    use anyhow::Context;
    use warp::Filter;

    type SyncImageBuffer = std::sync::Arc<tokio::sync::Mutex<ImageBuffer>>;

    pub struct ImageBuffer {
        client: reqwest::Client,
        images: std::collections::HashMap<String, Vec<u8>>,
    }

    impl ImageBuffer {
        pub fn new() -> Self {
            Self {
                client: Default::default(),
                images: Default::default(),
            }
        }
    }

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        image_url: String,
    }

    pub fn handle(
        args: args::SyncArgs,
        image_buffer: SyncImageBuffer,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            image_buffer: SyncImageBuffer,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit image buffer endpoint {:?}", request);

            let mut image_buffer = image_buffer.lock().await;

            let image = match image_buffer.images.get(&request.image_url) {
                Some(image) => {
                    tracing::debug!("use cached image {:?}", request.image_url);
                    image
                }
                None => {
                    tracing::info!("download new image {:?}", request.image_url);
                    let image = download_image(
                        &image_buffer.client,
                        &request.image_url,
                        args.image_width,
                        args.image_height,
                    )
                    .await
                    .map_err(reject::to)?;

                    tracing::debug!("create new cached image {:?}", request.image_url);
                    image_buffer
                        .images
                        .entry(request.image_url)
                        .or_insert(image)
                }
            };

            let response = warp::http::Response::builder()
                .status(200)
                .header("Content-Type", "image/jpeg")
                .body(image.clone())
                .context("failed to build http response to reply")
                .map_err(reject::to)?;

            Ok(response)
        }

        warp::path("image-get")
            .and(warp::get())
            .and(warp::query::<Request>())
            .map(move |request| (request, args.clone(), image_buffer.clone()))
            .untuple_one()
            .and_then(handle)
    }

    async fn download_image(
        client: &reqwest::Client,
        image_url: &str,
        image_width: u32,
        image_height: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let response = client
            .get(image_url)
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
            .resize_to_fill(image_width, image_height, image::imageops::CatmullRom)
            .write_to(&mut buf, image::ImageFormat::WebP)
            .context("failed to encode into webp from image")?;
        let bytes = buf.into_inner();
        tracing::info!("encoded image size {:?} bytes", bytes.len());

        Ok(bytes)
    }
}

mod image_index {
    use crate::state;
    use warp::Filter;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        duration_secs: f32,
        image_urls: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
    }

    pub fn handle(
        state: state::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(state: state::SyncState) -> impl warp::Reply {
            tracing::info!("hit image index api");

            let state = state.lock().await;

            let response = Response {
                duration_secs: state.duration_secs,
                image_urls: state.image_urls.iter().cloned().collect::<Vec<_>>(),
                image_url: state.image_url.clone(),
            };

            warp::reply::json(&response)
        }

        warp::path("image-index")
            .and(warp::get())
            .map(move || state.clone())
            .then(handle)
    }
}

mod image_modify {
    use crate::{args, reject, state};
    use warp::Filter;

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        duration_secs: Option<f32>,
        image_url: Option<String>,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: state::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: state::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit image modify endpoint {:?}", request);

            let mut state = state.lock().await;

            if let Some(image) = request.image_url {
                tracing::info!("set current image url {:?}", image);
                state.image_url = Some(image);
            }

            if let Some(duration) = request.duration_secs {
                tracing::info!("set duration {:?}", duration);
                state.duration_secs = duration;
            }

            state
                .write_to_file(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("image-modify")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod image_create {
    use crate::{args, reject, state};
    use warp::Filter;

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        image_url: String,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: state::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: state::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit image create endpoint {:?}", request);

            let mut state = state.lock().await;

            tracing::info!("insert new image url {:?}", request.image_url);
            state.image_urls.insert(request.image_url);

            state
                .write_to_file(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("image-create")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod image_delete {
    use crate::{args, reject, state};
    use warp::Filter;

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        image_url: String,
    }

    pub fn handle(
        args: args::SyncArgs,
        state: state::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: state::SyncState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            tracing::info!("hit image delete endpoint {:?}", request);

            let mut state = state.lock().await;

            tracing::info!("remove image url {:?}", request.image_url);
            state.image_urls.remove(&request.image_url);

            state
                .write_to_file(&args.state_filepath)
                .await
                .map_err(reject::to)?;

            Ok(warp::http::StatusCode::OK)
        }

        warp::path("image-delete")
            .and(warp::post())
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

    #[derive(Debug, Clone, serde::Serialize)]
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
