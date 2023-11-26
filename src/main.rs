use std::collections::HashSet;

use chrono::prelude::*;
use clap::Parser;
use futures::prelude::*;
use rand::prelude::*;
use serde::*;
use warp::Filter;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let safe_args = std::sync::Arc::new(args);

    let state = read_state(&safe_args.state_filepath)
        .await
        .unwrap_or_default();
    let safe_state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let filter = warp::path("api")
        .and(
            polling::handle(safe_state.clone())
                .or(pic_list::handle(safe_state.clone()))
                .or(pic_patch::handle(safe_args.clone(), safe_state.clone()))
                .or(pic_push::handle(safe_args.clone(), safe_state.clone()))
                .or(pic_pop::hanle(safe_args.clone(), safe_state.clone())),
        )
        .or(dist(
            safe_args.dist_dirpath.clone(),
            safe_args.dist_filepath.clone(),
        ))
        .with(cors());

    tokio::spawn(async {
        safe_state_loop(safe_state).await;
    });

    println!("listening on {}", safe_args.address);
    warp::serve(filter).run(safe_args.address).await;
}

fn dist(
    dirpath: impl Into<std::path::PathBuf>,
    filepath: impl Into<std::path::PathBuf>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::fs::dir(dirpath).or(warp::fs::file(filepath))
}

fn cors() -> warp::cors::Cors {
    warp::cors()
        .allow_any_origin()
        .allow_methods(["GET", "POST", "DELETE", "PATCH"])
        .allow_headers(["Content-Type"])
        .build()
}

type SafeArgs = std::sync::Arc<Args>;

#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Args {
    #[arg(long)]
    state_filepath: std::path::PathBuf,
    #[arg(long)]
    dist_dirpath: std::path::PathBuf,
    #[arg(long)]
    dist_filepath: std::path::PathBuf,
    #[arg(long)]
    address: std::net::SocketAddr,
}

type SafeState = std::sync::Arc<tokio::sync::Mutex<State>>;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    duration_secs: f32,
    urls: HashSet<String>,
    url: String,
}

async fn read_state(filepath: impl AsRef<std::path::Path>) -> Result<State, ErrorMessage> {
    let buf = tokio::fs::read(filepath)
        .await
        .map_err(ErrorMessage::from)?;
    serde_json::from_slice(&buf).map_err(ErrorMessage::from)
}

async fn write_state(
    filepath: impl AsRef<std::path::Path>,
    state: &State,
) -> Result<(), ErrorMessage> {
    let buf = serde_json::to_vec(&state).map_err(ErrorMessage::from)?;
    tokio::fs::write(filepath, buf)
        .await
        .map_err(ErrorMessage::from)
}

async fn safe_state_loop(safe_state: SafeState) {
    let mut rng = StdRng::from_entropy();

    loop {
        let mut state = safe_state.lock().await;

        if let Some(url) = state.urls.iter().choose(&mut rng) {
            state.url = url.to_string();
        }

        let duration_secs = state.duration_secs;

        drop(state);

        tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
    }
}

mod polling {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        date_time: DateTime<Local>,
        url: String,
    }

    pub fn handle(
        safe_state: SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn ws_handle(ws: warp::ws::WebSocket, safe_state: SafeState) {
            let (mut tx, mut rx) = ws.split();

            while let Some(_) = rx.next().await {
                let state = safe_state.lock().await;

                let res = Response {
                    date_time: Local::now(),
                    url: state.url.clone(),
                };

                let msg = warp::ws::Message::text(serde_json::to_string(&res).unwrap());
                let _ = tx.send(msg).await;
            }
        }
        warp::path("polling")
            .and(warp::ws())
            .map(move |ws| (ws, safe_state.clone()))
            .untuple_one()
            .map(|ws: warp::ws::Ws, safe_state| ws.on_upgrade(|ws| ws_handle(ws, safe_state)))
    }
}

mod pic_list {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        duration_secs: f32,
        urls: Vec<String>,
        url: String,
    }

    pub fn handle(
        safe_state: SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(safe_state: SafeState) -> Result<impl warp::Reply, warp::Rejection> {
            let state = safe_state.lock().await;

            let res = Response {
                duration_secs: state.duration_secs,
                urls: state.urls.iter().cloned().collect::<Vec<_>>(),
                url: state.url.clone(),
            };

            Ok(warp::reply::json(&res))
        }
        warp::path("pic")
            .and(warp::get())
            .map(move || safe_state.clone())
            .and_then(handle)
    }
}

mod pic_patch {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: Option<String>,
        duration_secs: Option<f32>,
    }

    pub fn handle(
        safe_args: SafeArgs,
        safe_state: SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: SafeArgs,
            safe_state: SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            if let Some(url) = req.url {
                state.url = url;
            }

            if let Some(duration_secs) = req.duration_secs {
                state.duration_secs = duration_secs;
            }

            write_state(&safe_args.state_filepath, &state).await?;
            Ok(warp::http::StatusCode::OK)
        }
        warp::path("pic")
            .and(warp::patch())
            .and(warp::body::json())
            .map(move |req| (req, safe_args.clone(), safe_state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod pic_push {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        safe_args: SafeArgs,
        safe_state: SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: SafeArgs,
            safe_state: SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            state.urls.insert(req.url);

            write_state(&safe_args.state_filepath, &state).await?;
            Ok(warp::http::StatusCode::OK)
        }
        warp::path("pic")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |req| (req, safe_args.clone(), safe_state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod pic_pop {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn hanle(
        safe_args: SafeArgs,
        safe_state: SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: SafeArgs,
            safe_state: SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            state.urls.remove(&req.url);

            write_state(&safe_args.state_filepath, &state).await?;
            Ok(warp::http::StatusCode::OK)
        }

        warp::path("pic")
            .and(warp::delete())
            .and(warp::body::json())
            .map(move |req| (req, safe_args.clone(), safe_state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

struct ErrorMessage {
    message: String,
}

impl std::fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl<E: std::error::Error> From<E> for ErrorMessage {
    fn from(value: E) -> Self {
        ErrorMessage {
            message: value.to_string(),
        }
    }
}

impl warp::reject::Reject for ErrorMessage {}
