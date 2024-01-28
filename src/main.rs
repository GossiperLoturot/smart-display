use std::*;

use clap::Parser;
use warp::Filter;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let safe_args = std::sync::Arc::new(args);

    let filepath = &safe_args.server_filepath;
    let server_state = match server::ServerState::read(filepath).await {
        Ok(server_state) => {
            println!("open server state file {:?}.", filepath);
            server_state
        }
        Err(e) => {
            println!("can't open server state file {:?}.", filepath);
            println!("{:?}", e);
            println!("use default server state");
            server::ServerState::default()
        }
    };
    let safe_server_state = std::sync::Arc::new(tokio::sync::Mutex::new(server_state));

    let filter = warp::path("api")
        .and(
            polling::handle(safe_server_state.clone())
                .or(picture_index::handle(safe_server_state.clone()))
                .or(picture_apply::handle(
                    safe_args.clone(),
                    safe_server_state.clone(),
                ))
                .or(picture_create::handle(
                    safe_args.clone(),
                    safe_server_state.clone(),
                ))
                .or(picture_delete::handle(
                    safe_args.clone(),
                    safe_server_state.clone(),
                )),
        )
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
        server::ServerState::safe_loop(safe_server_state).await;
    });

    println!("listening on {}.", safe_args.address);
    warp::serve(filter).run(safe_args.address).await;
}

mod args {
    pub type SafeArgs = std::sync::Arc<Args>;

    #[derive(clap::Parser)]
    #[clap(
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        author = env!("CARGO_PKG_AUTHORS"),
        about = env!("CARGO_PKG_DESCRIPTION"),
    )]
    pub struct Args {
        #[arg(long)]
        pub server_filepath: std::path::PathBuf,
        #[arg(long)]
        pub dist_dirpath: std::path::PathBuf,
        #[arg(long)]
        pub dist_filepath: std::path::PathBuf,
        #[arg(long)]
        pub address: std::net::SocketAddr,
    }
}

mod server {
    use rand::prelude::*;

    use crate::error::ErrorMessage;

    pub type SafeServerState = std::sync::Arc<tokio::sync::Mutex<ServerState>>;

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ServerState {
        pub duration_secs: f32,
        pub url_set: std::collections::HashSet<String>,
        pub current_url: Option<String>,
    }

    impl ServerState {
        pub async fn read(filepath: impl AsRef<std::path::Path>) -> Result<Self, ErrorMessage> {
            let buf = tokio::fs::read(filepath)
                .await
                .map_err(ErrorMessage::from)?;
            serde_json::from_slice(&buf).map_err(ErrorMessage::from)
        }

        pub async fn write(
            &self,
            filepath: impl AsRef<std::path::Path>,
        ) -> Result<(), ErrorMessage> {
            let buf = serde_json::to_vec(self).map_err(ErrorMessage::from)?;
            tokio::fs::write(filepath, buf)
                .await
                .map_err(ErrorMessage::from)
        }

        pub async fn safe_loop(safe_state: SafeServerState) {
            let mut rng = StdRng::from_entropy();

            loop {
                let mut state = safe_state.lock().await;

                if let Some(url) = state.url_set.iter().choose(&mut rng) {
                    state.current_url = Some(url.to_string());
                }

                let duration_secs = state.duration_secs;

                drop(state);

                tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
            }
        }
    }

    impl Default for ServerState {
        fn default() -> Self {
            Self {
                duration_secs: 60.0,
                url_set: std::collections::HashSet::new(),
                current_url: None,
            }
        }
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
    }

    pub fn handle(
        safe_server_state: server::SafeServerState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn ws_handle(ws: warp::ws::WebSocket, safe_server_state: server::SafeServerState) {
            println!("connect websocket transaction.");

            let (mut tx, mut rx) = ws.split();

            while rx.next().await.is_some() {
                let server_state = safe_server_state.lock().await;

                let response = Response {
                    date_time: chrono::Local::now(),
                    url: server_state.current_url.clone(),
                };

                let message = warp::ws::Message::text(serde_json::to_string(&response).unwrap());
                let _ = tx.send(message).await;
            }
        }

        warp::path("polling")
            .and(warp::ws())
            .map(move |ws| (ws, safe_server_state.clone()))
            .untuple_one()
            .map(|ws: warp::ws::Ws, safe_server_state| {
                ws.on_upgrade(|ws| ws_handle(ws, safe_server_state))
            })
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
        safe_server_state: server::SafeServerState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            safe_server_state: server::SafeServerState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            println!("request picture index.");

            let server_state = safe_server_state.lock().await;

            let response = Response {
                duration_secs: server_state.duration_secs,
                urls: server_state.url_set.iter().cloned().collect::<Vec<_>>(),
                url: server_state.current_url.clone(),
            };

            Ok(warp::reply::json(&response))
        }

        warp::path("config")
            .and(warp::get())
            .map(move || safe_server_state.clone())
            .and_then(handle)
    }
}

mod picture_apply {
    use warp::Filter;

    use crate::{args, server};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        duration_secs: Option<f32>,
        url: Option<String>,
    }

    pub fn handle(
        safe_args: args::SafeArgs,
        safe_server_state: server::SafeServerState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            safe_args: args::SafeArgs,
            safe_server_state: server::SafeServerState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            println!("apply picture {:?}.", request.url);

            let mut server_state = safe_server_state.lock().await;

            if let Some(url) = request.url {
                server_state.current_url = Some(url);
            }

            if let Some(duration_secs) = request.duration_secs {
                server_state.duration_secs = duration_secs;
            }

            server_state.write(&safe_args.server_filepath).await?;
            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::patch())
            .and(warp::body::json())
            .map(move |request| (request, safe_args.clone(), safe_server_state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod picture_create {
    use warp::Filter;

    use crate::{args, server};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        safe_args: args::SafeArgs,
        safe_server_state: server::SafeServerState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            safe_args: args::SafeArgs,
            safe_server_state: server::SafeServerState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            println!("create picture {:?}.", request.url);

            let mut server_state = safe_server_state.lock().await;

            server_state.url_set.insert(request.url);

            server_state.write(&safe_args.server_filepath).await?;
            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |request| (request, safe_args.clone(), safe_server_state.clone()))
            .untuple_one()
            .and_then(handle)
    }
}

mod picture_delete {
    use warp::Filter;

    use crate::{args, server};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        url: String,
    }

    pub fn handle(
        safe_args: args::SafeArgs,
        safe_server_state: server::SafeServerState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            safe_args: args::SafeArgs,
            safe_server_state: server::SafeServerState,
        ) -> Result<impl warp::Reply, warp::Rejection> {
            println!("delete picture {:?}.", request.url);

            let mut server_state = safe_server_state.lock().await;

            server_state.url_set.remove(&request.url);

            server_state.write(&safe_args.server_filepath).await?;
            Ok(warp::http::StatusCode::OK)
        }

        warp::path("config")
            .and(warp::delete())
            .and(warp::body::json())
            .map(move |request| (request, safe_args.clone(), safe_server_state.clone()))
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
