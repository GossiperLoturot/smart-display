use clap::Parser;
use warp::Filter;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let args = std::sync::Arc::new(args);

    let filepath = &args.state_filepath;
    let state = match server::State::read(filepath).await {
        Ok(server_state) => {
            println!("open server state file {:?}.", filepath);
            server_state
        }
        Err(e) => {
            println!("can't open server state file {:?}.", filepath);
            println!("{:?}", e);
            println!("use default server state");
            server::State::default()
        }
    };
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(state));

    let filter = warp::path("api")
        .and(
            polling::handle(state.clone())
                .or(picture_index::handle(state.clone()))
                .or(picture_apply::handle(args.clone(), state.clone()))
                .or(picture_create::handle(args.clone(), state.clone()))
                .or(picture_delete::handle(args.clone(), state.clone())),
        )
        .or(warp::fs::dir(args.dist_dirpath.clone()))
        .or(warp::fs::file(args.dist_filepath.clone()))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(["GET", "POST", "DELETE", "PATCH"])
                .allow_headers(["Content-Type"])
                .build(),
        );

    tokio::spawn(async {
        server::State::run(state).await;
    });

    println!("listening on {}.", args.address);
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
    }
}

mod server {
    use anyhow::Context;
    use rand::prelude::*;

    pub type SyncState = std::sync::Arc<tokio::sync::Mutex<State>>;

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub duration_secs: f32,
        pub url_set: std::collections::HashSet<String>,
        pub current_url: Option<String>,
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

        pub async fn run(state: SyncState) {
            let mut rng = StdRng::from_entropy();
            let client = reqwest::Client::new();

            loop {
                let mut state = state.lock().await;

                match state.update(&mut rng, &client).await {
                    Ok(()) => println!("update state"),
                    Err(e) => println!("{}", e),
                }

                let duration_secs = state.duration_secs;

                drop(state);

                tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
            }
        }

        async fn update(
            &mut self,
            rng: &mut rand::rngs::StdRng,
            client: &reqwest::Client,
        ) -> anyhow::Result<()> {
            let url = self
                .url_set
                .iter()
                .choose(rng)
                .cloned()
                .context("failed to choose image url")?;

            let response = client
                .get(url)
                .send()
                .await
                .context("failed to http request")?;

            if !response
                .headers()
                .get("Content-Type")
                .map(|value| value.as_bytes().starts_with(b"image/"))
                .unwrap_or_default()
            {
                anyhow::bail!("invalid Content-Type");
            }

            let bytes = response
                .bytes()
                .await
                .context("failed to read http response body")?;

            let img =
                image::load_from_memory(&bytes).context("failed to load image from response")?;
            let col = compute_repr_color(&img.to_rgb8());

            let mut buf = std::io::Cursor::new(vec![]);
            img.write_to(&mut buf, image::ImageFormat::WebP)
                .context("failed to save webp from image")?;
            let vec = buf.into_inner();

            println!("{:?}, {}", col, vec.len());

            Ok(())
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                duration_secs: 60.0,
                url_set: std::collections::HashSet::new(),
                current_url: None,
            }
        }
    }

    fn compute_repr_color(img: &image::RgbImage) -> image::Rgb<u8> {
        let pixel_len = img.len() / 3;
        let r = img.iter().step_by(3);
        let g = img.iter().skip(1).step_by(3);
        let b = img.iter().skip(2).step_by(3);
        let avg_r = (r.fold(0, |acc, x| acc + *x as usize) / pixel_len) as u8;
        let avg_g = (g.fold(0, |acc, x| acc + *x as usize) / pixel_len) as u8;
        let avg_b = (b.fold(0, |acc, x| acc + *x as usize) / pixel_len) as u8;
        image::Rgb([avg_r, avg_g, avg_b])
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
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn ws_handle(ws: warp::ws::WebSocket, state: server::SyncState) {
            println!("connect websocket transaction.");

            let (mut tx, mut rx) = ws.split();

            while rx.next().await.is_some() {
                let state = state.lock().await;

                let response = Response {
                    date_time: chrono::Local::now(),
                    url: state.current_url.clone(),
                };

                let message = warp::ws::Message::text(serde_json::to_string(&response).unwrap());
                let _ = tx.send(message).await;
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
            println!("request picture index.");

            let state = state.lock().await;

            let response = Response {
                duration_secs: state.duration_secs,
                urls: state.url_set.iter().cloned().collect::<Vec<_>>(),
                url: state.current_url.clone(),
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

    use crate::{args, server};

    #[derive(serde::Deserialize)]
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
        ) -> impl warp::Reply {
            println!("apply picture {:?}.", request.url);

            let mut state = state.lock().await;

            if let Some(url) = request.url {
                println!("change current url");
                state.current_url = Some(url);
            }

            if let Some(duration_secs) = request.duration_secs {
                println!("change duration secs");
                state.duration_secs = duration_secs;
            }

            if state.write(&args.state_filepath).await.is_err() {
                println!("failed to write state");
            }

            warp::http::StatusCode::OK
        }

        warp::path("config")
            .and(warp::patch())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .then(handle)
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
        args: args::SyncArgs,
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: server::SyncState,
        ) -> impl warp::Reply {
            println!("create picture {:?}.", request.url);

            let mut state = state.lock().await;

            state.url_set.insert(request.url);

            if state.write(&args.state_filepath).await.is_err() {
                println!("failed to write state");
            }

            warp::http::StatusCode::OK
        }

        warp::path("config")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .then(handle)
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
        args: args::SyncArgs,
        state: server::SyncState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        async fn handle(
            request: Request,
            args: args::SyncArgs,
            state: server::SyncState,
        ) -> impl warp::Reply {
            println!("delete picture {:?}.", request.url);

            let mut state = state.lock().await;

            state.url_set.remove(&request.url);

            if state.write(&args.state_filepath).await.is_err() {
                println!("failed to write state");
            }

            warp::http::StatusCode::OK
        }

        warp::path("config")
            .and(warp::delete())
            .and(warp::body::json())
            .map(move |request| (request, args.clone(), state.clone()))
            .untuple_one()
            .then(handle)
    }
}
