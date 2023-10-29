use std::collections::HashSet;

use chrono::prelude::*;
use futures::prelude::*;
use rand::prelude::*;
use serde::*;
use warp::Filter;

const ADDR: [u8; 4] = [0, 0, 0, 0];
const PORT: u16 = 3000;

#[tokio::main]
async fn main() {
    let app_state = read_app_state().await.unwrap_or_default();
    let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(app_state));
    let filter = warp::path("api")
        .and(
            polling(app_state.clone())
                .or(pic_list(app_state.clone()))
                .or(pic_patch(app_state.clone()))
                .or(pic_push(app_state.clone()))
                .or(pic_pop(app_state.clone())),
        )
        .or(warp::fs::dir("dist"))
        .or(warp::fs::file("dist/index.html"))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(["GET", "POST", "DELETE", "PATCH"])
                .allow_headers(["Content-Type"]),
        );

    tokio::spawn(async {
        app_loop(app_state).await;
    });

    let addr = std::net::SocketAddr::from((ADDR, PORT));
    println!("Listening on {}", addr);
    warp::serve(filter).run(addr).await;
}

type SharedAppState = std::sync::Arc<tokio::sync::Mutex<AppState>>;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    duration_secs: f32,
    urls: HashSet<String>,
    url: String,
}

async fn read_app_state() -> Result<AppState, ErrorMessage> {
    let buf = tokio::fs::read("smart-display.json")
        .await
        .map_err(ErrorMessage::from)?;
    serde_json::from_slice(&buf).map_err(ErrorMessage::from)
}

async fn write_app_state(app_state: &AppState) -> Result<(), ErrorMessage> {
    let buf = serde_json::to_vec(&app_state).map_err(ErrorMessage::from)?;
    tokio::fs::write("smart-display.json", buf)
        .await
        .map_err(ErrorMessage::from)
}

async fn app_loop(app_state: SharedAppState) {
    let mut rng = StdRng::from_entropy();
    loop {
        let mut app_state = app_state.lock().await;
        if let Some(url) = app_state.urls.iter().choose(&mut rng) {
            app_state.url = url.to_string();
        }
        let duration_secs = app_state.duration_secs;
        drop(app_state);

        tokio::time::sleep(std::time::Duration::from_secs_f32(duration_secs)).await;
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PollingOut {
    date_time: DateTime<Local>,
    url: String,
}

fn polling(
    app_state: SharedAppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn ws_handle(ws: warp::ws::WebSocket, app_state: SharedAppState) {
        let (mut tx, mut rx) = ws.split();
        while let Some(_) = rx.next().await {
            let app_state = app_state.lock().await;
            let output = PollingOut {
                date_time: Local::now(),
                url: app_state.url.clone(),
            };
            let text = serde_json::to_string(&output).unwrap();
            let msg = warp::ws::Message::text(text);
            let _ = tx.send(msg).await;
        }
    }
    warp::path("polling")
        .and(warp::ws())
        .map(move |ws| (ws, app_state.clone()))
        .untuple_one()
        .map(|ws: warp::ws::Ws, app_state| ws.on_upgrade(|ws| ws_handle(ws, app_state)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PicListOut {
    duration_secs: f32,
    urls: Vec<String>,
    url: String,
}

fn pic_list(
    app_state: SharedAppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn handle(app_state: SharedAppState) -> Result<impl warp::Reply, warp::Rejection> {
        let app_state = app_state.lock().await;
        let output = PicListOut {
            duration_secs: app_state.duration_secs,
            urls: app_state.urls.iter().cloned().collect::<Vec<_>>(),
            url: app_state.url.clone(),
        };
        Ok(warp::reply::json(&output))
    }
    warp::path("pic")
        .and(warp::get())
        .map(move || app_state.clone())
        .and_then(handle)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PicPatchIn {
    url: Option<String>,
    duration_secs: Option<f32>,
}

fn pic_patch(
    app_state: SharedAppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn handle(
        input: PicPatchIn,
        app_state: SharedAppState,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut app_state = app_state.lock().await;
        if let Some(url) = input.url {
            app_state.url = url;
        }
        if let Some(duration_secs) = input.duration_secs {
            app_state.duration_secs = duration_secs;
        }
        write_app_state(&app_state).await?;
        Ok(warp::http::StatusCode::OK)
    }
    warp::path("pic")
        .and(warp::patch())
        .and(warp::body::json())
        .map(move |input| (input, app_state.clone()))
        .untuple_one()
        .and_then(handle)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PicPushIn {
    url: String,
}

fn pic_push(
    app_state: SharedAppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn handle(
        input: PicPushIn,
        app_state: SharedAppState,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut app_state = app_state.lock().await;
        app_state.urls.insert(input.url);
        write_app_state(&app_state).await?;
        Ok(warp::http::StatusCode::OK)
    }
    warp::path("pic")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |input| (input, app_state.clone()))
        .untuple_one()
        .and_then(handle)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PicPopIn {
    url: String,
}

fn pic_pop(
    app_state: SharedAppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    async fn handle(
        input: PicPopIn,
        app_state: SharedAppState,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut app_state = app_state.lock().await;
        app_state.urls.remove(&input.url);
        write_app_state(&app_state).await?;
        Ok(warp::http::StatusCode::OK)
    }
    warp::path("pic")
        .and(warp::delete())
        .and(warp::body::json())
        .map(move |input| (input, app_state.clone()))
        .untuple_one()
        .and_then(handle)
}

#[derive(Debug)]
struct ErrorMessage {
    message: String,
}

impl<E: std::error::Error> From<E> for ErrorMessage {
    fn from(value: E) -> Self {
        ErrorMessage {
            message: value.to_string(),
        }
    }
}

impl warp::reject::Reject for ErrorMessage {}
