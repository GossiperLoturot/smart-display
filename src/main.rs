use std::*;

use clap::Parser;
use warp::Filter;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let safe_args = std::sync::Arc::new(args);

    let state = state::read_state(&safe_args.state_filepath)
        .await
        .unwrap_or_default();
    let safe_state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let cache = cache::read_cache(&safe_args.cache_filepath)
        .await
        .unwrap_or_default();
    let safe_cache = std::sync::Arc::new(tokio::sync::Mutex::new(cache));

    let filter = warp::path("api")
        .and(
            polling::handle(safe_state.clone())
                .or(pic_list::handle(safe_state.clone()))
                .or(pic_patch::handle(safe_args.clone(), safe_state.clone()))
                .or(pic_push::handle(safe_args.clone(), safe_state.clone()))
                .or(pic_pop::hanle(safe_args.clone(), safe_state.clone()))
                .or(pic_cache::hanle(safe_args.clone(), safe_cache.clone())),
        )
        .or(warp::path("cache").and(warp::fs::dir(safe_args.cache_dirpath.clone())))
        .or(warp::fs::dir(safe_args.dist_dirpath.clone()))
        .or(warp::fs::file(safe_args.dist_filepath.clone()))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(["GET", "POST", "DELETE", "PATCH"])
                .allow_headers(["Content-Type"])
                .build(),
        );

    tokio::spawn(async {
        state::safe_state_loop(safe_state).await;
    });

    println!("listening on {}", safe_args.address);
    warp::serve(filter).run(safe_args.address).await;
}

mod args {
    pub type SafeArgs = std::sync::Arc<Args>;

    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long)]
        pub state_filepath: std::path::PathBuf,
        #[arg(long)]
        pub dist_dirpath: std::path::PathBuf,
        #[arg(long)]
        pub dist_filepath: std::path::PathBuf,
        #[arg(long)]
        pub cache_dirpath: std::path::PathBuf,
        #[arg(long)]
        pub cache_filepath: std::path::PathBuf,
        #[arg(long)]
        pub address: std::net::SocketAddr,
    }
}

mod state {
    use rand::prelude::*;

    use crate::error::ErrorMessage;

    pub type SafeState = std::sync::Arc<tokio::sync::Mutex<State>>;

    #[derive(Default, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub duration_secs: f32,
        pub urls: std::collections::HashSet<String>,
        pub url: String,
    }

    pub async fn read_state(filepath: impl AsRef<std::path::Path>) -> Result<State, ErrorMessage> {
        let buf = tokio::fs::read(filepath)
            .await
            .map_err(ErrorMessage::from)?;
        serde_json::from_slice(&buf).map_err(ErrorMessage::from)
    }

    pub async fn write_state(
        filepath: impl AsRef<std::path::Path>,
        state: &State,
    ) -> Result<(), ErrorMessage> {
        let buf = serde_json::to_vec(&state).map_err(ErrorMessage::from)?;
        tokio::fs::write(filepath, buf)
            .await
            .map_err(ErrorMessage::from)
    }

    pub async fn safe_state_loop(safe_state: SafeState) {
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
}

mod cache {
    use crate::error::ErrorMessage;

    pub type SafeCache = std::sync::Arc<tokio::sync::Mutex<Cache>>;

    #[derive(Default, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Cache {
        pub map: std::collections::HashMap<String, String>,
    }

    pub async fn read_cache(filepath: impl AsRef<std::path::Path>) -> Result<Cache, ErrorMessage> {
        let buf = tokio::fs::read(filepath)
            .await
            .map_err(ErrorMessage::from)?;
        serde_json::from_slice(&buf).map_err(ErrorMessage::from)
    }

    pub async fn write_cache(
        filepath: impl AsRef<std::path::Path>,
        cache: &Cache,
    ) -> Result<(), ErrorMessage> {
        let buf = serde_json::to_vec(&cache).map_err(ErrorMessage::from)?;
        tokio::fs::write(filepath, buf)
            .await
            .map_err(ErrorMessage::from)
    }
}

mod polling {
    use futures::{SinkExt, StreamExt};
    use warp::Filter;

    use crate::state;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        date_time: chrono::DateTime<chrono::Local>,
        url: String,
    }

    pub fn handle(
        safe_state: state::SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn ws_handle(ws: warp::ws::WebSocket, safe_state: state::SafeState) {
            let (mut tx, mut rx) = ws.split();

            while let Some(_) = rx.next().await {
                let state = safe_state.lock().await;

                let res = Response {
                    date_time: chrono::Local::now(),
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
    use warp::Filter;

    use crate::state;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        duration_secs: f32,
        urls: Vec<String>,
        url: String,
    }

    pub fn handle(
        safe_state: state::SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(safe_state: state::SafeState) -> Result<impl warp::Reply, warp::Rejection> {
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
    use warp::Filter;

    use crate::{args, state};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: Option<String>,
        duration_secs: Option<f32>,
    }

    pub fn handle(
        safe_args: args::SafeArgs,
        safe_state: state::SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: args::SafeArgs,
            safe_state: state::SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            if let Some(url) = req.url {
                state.url = url;
            }

            if let Some(duration_secs) = req.duration_secs {
                state.duration_secs = duration_secs;
            }

            state::write_state(&safe_args.state_filepath, &state).await?;
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
    use warp::Filter;

    use crate::{args, state};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        safe_args: args::SafeArgs,
        safe_state: state::SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: args::SafeArgs,
            safe_state: state::SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            state.urls.insert(req.url);

            state::write_state(&safe_args.state_filepath, &state).await?;
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
    use warp::Filter;

    use crate::{args, state};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn hanle(
        safe_args: args::SafeArgs,
        safe_state: state::SafeState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: args::SafeArgs,
            safe_state: state::SafeState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut state = safe_state.lock().await;

            state.urls.remove(&req.url);

            state::write_state(&safe_args.state_filepath, &state).await?;
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

mod pic_cache {
    use base64::prelude::*;
    use rand::prelude::*;
    use warp::Filter;

    use crate::{args, cache, error::ErrorMessage};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        id: String,
    }

    pub fn hanle(
        safe_args: args::SafeArgs,
        safe_cache: cache::SafeCache,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            req: Request,
            safe_args: args::SafeArgs,
            safe_cache: cache::SafeCache,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            let mut cache = safe_cache.lock().await;

            let url = req.url;
            if !cache.map.contains_key(&url) {
                let mut bytes = [0; 64];
                StdRng::from_entropy().fill_bytes(&mut bytes);
                let id = BASE64_URL_SAFE.encode(bytes);

                let bytes = reqwest::get(&url)
                    .await
                    .map_err(ErrorMessage::from)?
                    .bytes()
                    .await
                    .map_err(ErrorMessage::from)?;

                let filepath = &safe_args.cache_dirpath.join(&id);
                tokio::fs::write(filepath, bytes)
                    .await
                    .map_err(ErrorMessage::from)?;

                cache.map.insert(url.clone(), id);
                cache::write_cache(&safe_args.cache_filepath, &cache).await?;
            }

            let id = cache.map.get(&url).unwrap().clone();
            let res = Response { id };

            Ok(warp::reply::json(&res))
        }

        warp::path("pic_cache")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |req| (req, safe_args.clone(), safe_cache.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod error {
    pub struct ErrorMessage {
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
}
