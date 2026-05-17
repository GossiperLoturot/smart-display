use anyhow::Context;
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
        #[arg(long, default_value = "sqlite:state.db")]
        pub database_url: String,
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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("starting server.");

    let args = args::Args::parse();
    let shared_args = std::sync::Arc::new(args);

    let state = state::State::try_new(&shared_args).await?;
    let shared_state = std::sync::Arc::new(state);

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
            let state = shared_state.clone();
            move |request| image_modify::handle(state, request)
        }))
        .route("/image-create", axum::routing::post({
            let args = shared_args.clone();
            let state = shared_state.clone();
            move |request| image_create::handle(args, state, request)
        }))
        .route("/image-delete", axum::routing::post({
            let state = shared_state.clone();
            move |request| image_delete::handle(state, request)
        }))
        .route("/image-get", axum::routing::get({
            let state = shared_state.clone();
            move |request| image_get::handle(state, request)
        }))
        .fallback_service(serve_dir.fallback(serve_idx))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024));

    tracing::info!("start background state routine.");
    tokio::spawn(state::update_state(shared_state.clone()));

    tracing::info!("listening on {}", shared_args.address);
    let listener = tokio::net::TcpListener::bind(&shared_args.address)
        .await
        .context("failed to bind tcp listener.")?;
    axum::serve(listener, app)
        .await
        .context("failed to execute server.")?;
    Ok(())
}

mod state {
    use std::str::FromStr as _;
    use anyhow::Context as _;
    use crate::args;

    #[derive(Debug, Clone)]
    pub struct State {
        client: sqlx::SqlitePool
    }

    pub type SharedState = std::sync::Arc<State>;

    impl State {
        pub async fn try_new(args: &args::Args) -> anyhow::Result<Self> {
            tracing::info!("connecting to database: {}", args.database_url);
            let opt = sqlx::sqlite::SqliteConnectOptions::from_str(&args.database_url)?
                .foreign_keys(true)
                .create_if_missing(true);
            let client = sqlx::SqlitePool::connect_with(opt).await?;
            tracing::info!("successfully connected to database.");

            // migration
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    duration_secs REAL NOT NULL,
                    current_image_id TEXT,
                    FOREIGN KEY(current_image_id) REFERENCES images(id) ON DELETE SET NULL
                )"
            )
                .execute(&client)
                .await
                .context("failed to create state table.")?;
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS images (
                    id TEXT PRIMARY KEY,
                    bytes BLOB NOT NULL,
                    created_at INTEGER NOT NULL
                )"
            )
                .execute(&client)
                .await
                .context("failed to create images table.")?;
            sqlx::query("INSERT OR IGNORE INTO state (id, duration_secs, current_image_id) VALUES (1, 60.0, NULL)")
                .execute(&client)
                .await
                .context("failed to initialize state.")?;
            tracing::info!("database migration completed.");

            Ok(State { client })
        }

        pub async fn duration_secs(&self) -> anyhow::Result<f64> {
            let row: (f64,) = sqlx::query_as("SELECT duration_secs FROM state WHERE id = 1")
                .fetch_one(&self.client)
                .await?;
            Ok(row.0)
        }

        pub async fn set_duration_secs(&self, value: f64) -> anyhow::Result<()> {
            sqlx::query("UPDATE state SET duration_secs = $1 WHERE id = 1")
                .bind(value)
                .execute(&self.client)
                .await?;
            Ok(())
        }

        pub async fn current_image_id(&self) -> anyhow::Result<Option<uuid::Uuid>> {
            let row: (Option<uuid::Uuid>,) = sqlx::query_as("SELECT current_image_id FROM state WHERE id = 1")
                .fetch_one(&self.client)
                .await?;
            Ok(row.0)
        }

        pub async fn set_current_image_id(&self, value: Option<uuid::Uuid>) -> anyhow::Result<()> {
            sqlx::query("UPDATE state SET current_image_id = $1 WHERE id = 1")
                .bind(value)
                .execute(&self.client)
                .await?;
            Ok(())
        }

        pub async fn image_ids(&self) -> anyhow::Result<Vec<uuid::Uuid>> {
            let rows: Vec<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM images ORDER BY created_at DESC")
                .fetch_all(&self.client)
                .await?;
            let image_ids = rows.into_iter().map(|(image_id,)| image_id).collect();
            Ok(image_ids)
        }

        pub async fn get_image(&self, image_id: uuid::Uuid) -> anyhow::Result<Vec<u8>> {
            let row: (Vec<u8>,) = sqlx::query_as("SELECT bytes FROM images WHERE id = $1")
                .bind(image_id)
                .fetch_one(&self.client)
                .await?;
            Ok(row.0)
        }

        pub async fn insert_image(&self, id: uuid::Uuid, bytes: Vec<u8>) -> anyhow::Result<()> {
            sqlx::query("INSERT INTO images (id, bytes, created_at) VALUES ($1, $2, strftime('%s', 'now'))")
                .bind(id)
                .bind(bytes)
                .execute(&self.client)
                .await?;
            Ok(())
        }

        pub async fn remove_image(&self, id: uuid::Uuid) -> anyhow::Result<()> {
            sqlx::query("DELETE FROM images WHERE id = $1")
                .bind(id)
                .execute(&self.client)
                .await?;
            Ok(())
        }
    }

    pub async fn update_state(state: SharedState) {
        let mut rng = rand::make_rng::<rand::rngs::StdRng>();

        loop {
            // attempt to fetch image ids.
            let image_ids = match state.image_ids().await {
                Ok(image_ids) => image_ids,
                Err(e) => {
                    tracing::error!("failed to fetch image ids: {:#}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            // choose random image.
            match rand::seq::IteratorRandom::choose(image_ids.into_iter(), &mut rng) {
                Some(image_id) => {
                    tracing::info!("successfully updated current image id: {}", image_id);
                    state.set_current_image_id(Some(image_id)).await.unwrap();
                }
                None => {
                    tracing::warn!("failed to update image id.");
                }
            }

            // attempt to fetch duration secs.
            let duration_secs = match state.duration_secs().await {
                Ok(duration_secs) => duration_secs,
                Err(e) => {
                    tracing::error!("failed to fetch duration secs: {:#}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            // sleep until next time.
            tokio::time::sleep(std::time::Duration::from_secs_f64(duration_secs)).await;
        }
    }
}

mod polling {
    use anyhow::Context as _;
    use crate::{reject, state};

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        #[serde(with = "time::serde::iso8601")]
        date_time: time::OffsetDateTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        current_image_id: Option<uuid::Uuid>,
    }

    pub async fn handle(state: state::SharedState) -> anyhow::Result<axum::Json<Response>, reject::Rejection> {
        // tracing::debug!("handling polling request.");

        let date_time = time::OffsetDateTime::now_local()
            .context("failed to get local time.")
            .map_err(reject::Rejection)?;
        let current_image_id = state.current_image_id().await.map_err(reject::Rejection)?;
        let response = Response { date_time, current_image_id };

        Ok(axum::Json(response))
    }
}

mod image_index {
    use crate::{reject, state};

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        duration_secs: f64,
        image_ids: Vec<uuid::Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        current_image_id: Option<uuid::Uuid>,
    }

    pub async fn handle(state: state::SharedState) -> anyhow::Result<axum::Json<Response>, reject::Rejection> {
        tracing::info!("handling image index request.");

        let duration_secs = state.duration_secs().await.map_err(reject::Rejection)?;
        let current_image_id = state.current_image_id().await.map_err(reject::Rejection)?;
        let image_ids = state.image_ids().await.map_err(reject::Rejection)?;
        let response = Response {
            duration_secs,
            current_image_id,
            image_ids,
        };

        Ok(axum::Json(response))
    }
}

mod image_get {
    use anyhow::Context as _;
    use crate::{reject, state};

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        image_id: uuid::Uuid,
    }

    pub async fn handle(
        state: state::SharedState,
        request: axum::extract::Query<Request>,
    ) -> Result<axum::response::Response, reject::Rejection> {
        tracing::info!("handling image get request: {:?}", request);

        let image = state.get_image(request.image_id).await.map_err(reject::Rejection)?;

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
    use anyhow::Context as _;
    use crate::{args, reject, state};

    pub async fn handle(
        args: args::SharedArgs,
        state: state::SharedState,
        mut request: axum::extract::Multipart,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("handling image create request: {:?}", request);

        let mut image_bytes = None;
        while let Some(field) = request.next_field()
            .await
            .context("failed to read multipart field.")
            .map_err(reject::Rejection)?
        {
            if field.name() == Some("image") {
                let data = field.bytes()
                    .await
                    .context("failed to read multipart data.")
                    .map_err(reject::Rejection)?;
                image_bytes = Some(data);
            }
        }
        let image_bytes = image_bytes
            .context("failed to find to image.")
            .map_err(reject::Rejection)?;

        let image_width = args.image_width;
        let image_height = args.image_height;
        let image_bytes = tokio::task::spawn_blocking(move || {
            let image = image::load_from_memory(&image_bytes)?;
            let mut buf = std::io::Cursor::new(vec![]);
            image
                .resize_to_fill(image_width, image_height, image::imageops::CatmullRom)
                .write_to(&mut buf, image::ImageFormat::WebP)?;
            Ok::<_, anyhow::Error>(buf.into_inner())
        })
            .await
            .context("failed to join blocking task.")
            .map_err(reject::Rejection)?
            .context("failed to process image.")
            .map_err(reject::Rejection)?;
        tracing::info!("successfully registered image: {} bytes", image_bytes.len());

        let image_id = uuid::Uuid::new_v4();
        state.insert_image(image_id, image_bytes).await.map_err(reject::Rejection)?;

        Ok(axum::http::StatusCode::OK)
    }
}

mod image_modify {
    use crate::{reject, state};

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        duration_secs: Option<f64>,
        current_image_id: Option<uuid::Uuid>,
    }

    pub async fn handle(
        state: state::SharedState,
        request: axum::Json<Request>,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("handling image modify request: {:?}", request);

        if let Some(current_image_id) = request.current_image_id {
            tracing::info!("set current image id {}", current_image_id);
            state.set_current_image_id(Some(current_image_id)).await.map_err(reject::Rejection)?;
        }

        if let Some(duration) = request.duration_secs {
            tracing::info!("set duration {}", duration);
            state.set_duration_secs(duration).await.map_err(reject::Rejection)?;
        }

        Ok(axum::http::StatusCode::OK)
    }
}

mod image_delete {
    use crate::{reject, state};

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        image_id: uuid::Uuid,
    }

    pub async fn handle(
        state: state::SharedState,
        request: axum::Json<Request>,
    ) -> Result<axum::http::StatusCode, reject::Rejection> {
        tracing::info!("handling image delete request: {:?}", request);

        state.remove_image(request.image_id).await.map_err(reject::Rejection)?;
        tracing::info!("successfully deleted image id: {}", request.image_id);

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
            tracing::error!("error response: {:#}", self.0);

            let status_code = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
            let response = ErrorResponse {
                status_code: status_code.as_u16(),
                message: "internal server error".to_string(),
            };
            (status_code, axum::Json(response)).into_response()
        }
    }
}
