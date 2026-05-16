use clap::Parser;

mod args {
    #[derive(Debug, Clone, clap::Parser)]
    #[clap(
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        author = env!("CARGO_PKG_AUTHORS"),
        about = env!("CARGO_PKG_DESCRIPTION"),
    )]
    pub struct Args {
        #[arg(long, default_value = "./state.bin")]
        pub state_filepath: std::path::PathBuf,
        #[arg(long, default_value = "0.0.0.0:50822")]
        pub address: std::net::SocketAddr,
        #[arg(long, default_value = "./html")]
        pub html: std::path::PathBuf,
        #[arg(long, default_value = "780")]
        pub image_width: u32,
        #[arg(long, default_value = "460")]
        pub image_height: u32,
    }

    pub type SharedArgs = std::sync::Arc<Args>;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = args::Args::parse();
    let shared_args = std::sync::Arc::new(args);

    let state = match state::state_from_fs(&shared_args.state_filepath).await {
        Ok(state) => {
            tracing::info!("load state file {:?}", shared_args.state_filepath);
            tracing::info!("use loaded state file");
            state
        }
        Err(err) => {
            tracing::warn!("failed to load state file {:?}", err);
            tracing::info!("use default state file");
            state::State::default()
        }
    };
    let shared_state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let serve_dir = tower_http::services::ServeDir::new(shared_args.html.clone());
    let serve_idx = tower_http::services::ServeFile::new(shared_args.html.join("index.html"));
    let app = axum::Router::new()
        .route("/polling", axum::routing::get({
            let state = shared_state.clone();
            move || polling::handle(state)
        }))
        .route("/image-index", axum::routing::get({
            let state = shared_state.clone();
            move || image_index::handle(state)
        }))
        .route("/image-modify", axum::routing::post({
            let args = shared_args.clone();
            let state = shared_state.clone();
            move |request| image_modify::handle(args, state, request)
        }))
        .route("/image-create", axum::routing::post({
            let args = shared_args.clone();
            let state = shared_state.clone();
            move |request| image_create::handle(args, state, request)
        }))
        .route("/image-delete", axum::routing::post({
            let args = shared_args.clone();
            let state = shared_state.clone();
            move |request| image_delete::handle(args, state, request)
        }))
        .route("/image-get", axum::routing::get({
            let state = shared_state.clone();
            move |request| image_get::handle(state, request)
        }))
        .fallback_service(serve_dir.fallback(serve_idx));

    tracing::info!("start background state routine");
    tokio::spawn(state::update_state(shared_state.clone()));

    tracing::info!("start listening {:?}", shared_args.address);
    let listener = tokio::net::TcpListener::bind(&shared_args.address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod state {
    use anyhow::Context;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub duration_secs: f32,
        pub image_kvs: std::collections::HashMap<uuid::Uuid, Vec<u8>>,
        #[serde(skip)]
        pub image_key: Option<uuid::Uuid>,
    }

    pub type SharedState = std::sync::Arc<tokio::sync::Mutex<State>>;

    impl Default for State {
        fn default() -> Self {
            Self {
                duration_secs: 60.0,
                image_kvs: Default::default(),
                image_key: Default::default(),
            }
        }
    }

    pub async fn state_from_fs(path: impl AsRef<std::path::Path>) -> anyhow::Result<State> {
        let buf = tokio::fs::read(path).await.context("failed to read state from file")?;
        let state = postcard::from_bytes(&buf).context("failed to deserialize state from bin")?;
        Ok(state)
    }

    pub async fn state_to_fs(state: &State, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let buf = postcard::to_allocvec(state).context("failed to serialize state to bin")?;
        tokio::fs::write(path, buf).await.context("failed to write state to file")?;
        Ok(())
    }

    pub async fn update_state(state: SharedState) {
        let mut rng = rand::make_rng::<rand::rngs::StdRng>();

        loop {
            let mut state = state.lock().await;

            match rand::seq::IteratorRandom::choose(state.image_kvs.keys(), &mut rng) {
                Some(image_key) => {
                    tracing::debug!("success to set image key {:?}", image_key);
                    state.image_key = Some(*image_key);
                }
                None => {
                    tracing::debug!("failed to set image key");
                }
            }

            let duration_secs = std::time::Duration::from_secs_f32(state.duration_secs);

            drop(state);

            tokio::time::sleep(duration_secs).await;
        }
    }
}

mod polling {
    use crate::state;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        #[serde(with = "time::serde::iso8601")]
        date_time: time::OffsetDateTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_key: Option<uuid::Uuid>,
    }

    pub async fn handle(shared_state: state::SharedState) -> axum::Json<Response> {
        let state = shared_state.lock().await;

        let date_time = time::OffsetDateTime::now_local()
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        let image_key = state.image_key;
        let response = Response { date_time, image_key };

        axum::Json(response)
    }
}

mod image_index {
    use crate::state;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        duration_secs: f32,
        image_keys: Vec<uuid::Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_key: Option<uuid::Uuid>,
    }

    pub async fn handle(
        state: state::SharedState,
    ) -> axum::Json<Response> {
        tracing::info!("hit image index api");

        let state = state.lock().await;

        let response = Response {
            duration_secs: state.duration_secs,
            image_keys: state.image_kvs.keys().copied().collect::<Vec<_>>(),
            image_key: state.image_key,
        };

        axum::Json(response)
    }
}

mod image_get {
    use crate::{reject, state};
    use anyhow::Context;

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        image_key: uuid::Uuid,
    }

    pub async fn handle(
        shared_state: state::SharedState,
        request: axum::Form<Request>,
    ) -> Result<axum::response::Response, reject::Rejection> {
        tracing::info!("hit image buffer endpoint {:?}", request);

        let state = shared_state.lock().await;

        let image = state.image_kvs
            .get(&request.image_key)
            .context("failed to get image")
            .map_err(reject::Rejection)?;

        let response = axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "image/webp")
            .body(axum::body::Body::from(image.clone()))
            .context("failed to build http response to reply")
            .map_err(reject::Rejection)?;
        Ok(response)
    }
}

mod image_create {
    use crate::{args, reject, state};
    use anyhow::Context;

    pub async fn handle(
        args: args::SharedArgs,
        state: state::SharedState,
        mut request: axum::extract::Multipart,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("hit image create endpoint {:?}", request);

        let mut state = state.lock().await;

        tracing::info!("insert new image");
        let mut image_bytes = None;
        while let Some(field) = request.next_field()
            .await
            .context("failed to read multipart field")
            .map_err(reject::Rejection)?
        {
            if field.name() == Some("image") {
                let data = field.bytes()
                    .await
                    .context("failed to read multipart data")
                    .map_err(reject::Rejection)?;
                image_bytes = Some(data);
            }
        }
        let image_bytes = image_bytes
            .context("failed to find to image")
            .map_err(reject::Rejection)?;
        tracing::info!("read image binary (size: {})", image_bytes.len());
        let image = image::load_from_memory(&image_bytes)
            .context("failed to load image from http response body")
            .map_err(reject::Rejection)?;
        let mut buf = std::io::Cursor::new(vec![]);
        image
            .resize_to_fill(args.image_width, args.image_height, image::imageops::CatmullRom)
            .write_to(&mut buf, image::ImageFormat::WebP)
            .context("failed to encode into webp from image")
            .map_err(reject::Rejection)?;
        let image_bytes = buf.into_inner();
        tracing::info!("encoded image size {:?} bytes", image_bytes.len());

        let image_key = uuid::Uuid::new_v4();
        state.image_kvs.insert(image_key, image_bytes);
        state::state_to_fs(&state, &args.state_filepath)
            .await
            .map_err(reject::Rejection)?;

        Ok(axum::http::StatusCode::OK)
    }
}

mod image_modify {
    use crate::{args, reject, state};

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        duration_secs: Option<f32>,
        image_key: Option<uuid::Uuid>,
    }

    pub async fn handle(
        args: args::SharedArgs,
        state: state::SharedState,
        request: axum::Json<Request>,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("hit image modify endpoint {:?}", request);

        let mut state = state.lock().await;

        if let Some(image_key) = request.image_key {
            tracing::info!("set current image key {:?}", image_key);
            state.image_key = Some(image_key);
        }

        if let Some(duration) = request.duration_secs {
            tracing::info!("set duration {:?}", duration);
            state.duration_secs = duration;
        }

        state::state_to_fs(&state, &args.state_filepath)
            .await
            .map_err(reject::Rejection)?;

        Ok(axum::http::StatusCode::OK)
    }
}

mod image_delete {
    use crate::{args, reject, state};

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        image_key: uuid::Uuid,
    }

    pub async fn handle(
        args: args::SharedArgs,
        state: state::SharedState,
        request: axum::Json<Request>,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("hit image delete endpoint {:?}", request);

        let mut state = state.lock().await;

        tracing::info!("remove image key {:?}", request.image_key);
        state.image_kvs.remove(&request.image_key);

        state::state_to_fs(&state, &args.state_filepath)
            .await
            .map_err(reject::Rejection)?;

        Ok(axum::http::StatusCode::OK)
    }
}

mod reject {
    #[derive(Debug)]
    pub struct Rejection(pub anyhow::Error);

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ErrorResponse {
        status_code: u16,
        message: String,
    }

    impl axum::response::IntoResponse for Rejection {
        fn into_response(self) -> axum::response::Response {
            let status_code = axum::http::StatusCode::BAD_REQUEST;
            let response = ErrorResponse {
                status_code: status_code.as_u16(),
                message: self.0.to_string(),
            };
            (status_code, axum::Json(response)).into_response()
        }
    }
}
