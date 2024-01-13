use axum::{
    body::Body,
    extract::{Request, State},
    http::{uri::Uri, HeaderValue},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD as base64url;
use base64::{alphabet, engine, DecodeError, Engine};
use hyper::{Method, StatusCode};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

async fn proxy_handler(
    State(client): State<Client>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://localhost:5173{}", path_query);
    log::info!("Proxying request: {:?} to {:?}", req.uri(), uri);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    Ok(client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response())
}

#[derive(Debug, Serialize)]
struct User {
    id: String,
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
}
#[derive(Debug, Serialize)]
struct Credentials {
    challenge: String,
    user: User,
    // excludeCredentials: Vec<ExcludeCredential>,
}

async fn credentials() -> (StatusCode, Json<Credentials>) {
    //TODO random binary value
    let challenge = base64url.encode(b"hello world");
    let user_id = base64url.encode(b"123");
    let user = User {
        id: user_id,
        name: "SantaClaas".to_string(),
        display_name: "Claas".to_string(),
    };
    (StatusCode::OK, Json(Credentials { challenge, user }))
}
#[tokio::main]
async fn main() {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    // build our application with a single route
    let app = Router::new()
        // .route("/", get(|| async { "Hello, World!" }))
        // .route("/*key", get(proxy_handler))
        .route("/credentials", get(credentials))
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
        .with_state(client);
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
