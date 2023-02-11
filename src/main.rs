use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_kube::kube::{Kube, PodInfo, PodLogReq, PodStatus};
use kube::{Client, Config};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::log::info;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // initialize kube
    let config = Config::infer().await.unwrap();
    let client = Client::try_from(config).unwrap();
    let kube = Kube::from(client);

    // build our application with a route
    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/", get(root))
        .route("/:namespace/:pod_name/status", get(query_pod_status))
        .route("/:namespace/:pod_name/logs", get(query_pod_log))
        .route("/:namespace/:pod_name", delete(stop_pod))
        .route("/:namespace/pod", post(create_pod))
        .with_state(kube);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root(State(kube): State<Kube>) -> &'static str {
    "Hello, World!"
}

#[derive(Debug, Serialize)]
enum BaseResponseCode {
    OK,
    FAIL,
}
#[derive(Debug, Serialize)]
struct BaseResponse<D, E> {
    code: BaseResponseCode,
    data: Option<D>,
    err: Option<E>,
}

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

async fn create_pod(
    State(kube): State<Kube>,
    Path(namespace): Path<String>,
    Json(pod_info): Json<PodInfo>,
) -> impl IntoResponse {
    match kube.create_pod(&namespace, &pod_info).await {
        Ok(pod) => Json(BaseResponse {
            code: BaseResponseCode::OK,
            data: Some(pod),
            err: None,
        }),
        Err(e) => Json(BaseResponse {
            code: BaseResponseCode::FAIL,
            data: None,
            err: Some(format!("{}", e)),
        }),
    }
}

async fn query_pod_status(
    State(kube): State<Kube>,
    Path((namespace, pod_name)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("{},{}", namespace, pod_name);
    match kube.query_pod_status(&namespace, &pod_name).await {
        Ok(status) => Json(BaseResponse {
            code: BaseResponseCode::OK,
            data: Some(status),
            err: None,
        }),
        Err(e) => Json(BaseResponse {
            code: BaseResponseCode::FAIL,
            data: None,
            err: Some(format!("{}", e)),
        }),
    }
}

#[derive(Deserialize)]
struct PodLogQuery {
    tail: i64,
}
async fn query_pod_log(
    State(kube): State<Kube>,
    Path((namespace, pod_name)): Path<(String, String)>,
    query: Query<PodLogQuery>,
) -> impl IntoResponse {
    match kube
        .query_pod_logs(
            &namespace,
            &PodLogReq {
                pod_name,
                tail_lines: query.tail,
            },
        )
        .await
    {
        Ok(log) => Json(BaseResponse {
            code: BaseResponseCode::OK,
            data: Some(log),
            err: None,
        }),
        Err(e) => Json(BaseResponse {
            code: BaseResponseCode::FAIL,
            data: None,
            err: Some(format!("{}", e)),
        }),
    }
}

async fn stop_pod(
    State(kube): State<Kube>,
    Path((namespace, pod_name)): Path<(String, String)>,
) -> impl IntoResponse {
    match kube.stop_pod(&namespace, &pod_name).await {
        Ok(_) => Json(BaseResponse {
            code: BaseResponseCode::OK,
            data: Some(()),
            err: None,
        }),
        Err(e) => Json(BaseResponse {
            code: BaseResponseCode::FAIL,
            data: None,
            err: Some(format!("{}", e)),
        }),
    }
}
