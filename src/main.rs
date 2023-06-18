use hyper::client::connect::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, HeaderMap, Method, Request, Response, Server};
use hyper_tls::HttpsConnector;
// use ratelimit::Ratelimiter;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
const GAME_SERVER_URL: &str = "https://api.spacetraders.io/v2/";
pub mod ratelimit;
use ratelimit::Ratelimiter;

#[derive(Clone)]
struct AppContext {
    client: Client<HttpsConnector<HttpConnector>>,
    ratelimiter: Arc<Ratelimiter>,
}

async fn st_req(
    method: &Method,
    path: &String,
    headers: &HeaderMap,
    body: Body,
    client: Client<HttpsConnector<HttpConnector>>,
    rl: Arc<Ratelimiter>,
) -> Response<Body> {
    let mut r = Request::builder()
        .method(method)
        .uri(GAME_SERVER_URL.to_owned() + path);
    if headers.contains_key(AUTHORIZATION) {
        r = r.header(AUTHORIZATION, headers.get(AUTHORIZATION).unwrap());
    }
    if headers.contains_key(CONTENT_TYPE) {
        r = r.header(CONTENT_TYPE, headers.get(CONTENT_TYPE).unwrap());
    }
    let req = r.body(body).unwrap();
    loop {
        if let Err(sleep) = rl.try_wait() {
            tokio::time::sleep(sleep).await;
        } else{
            break;
        }
    }
    let response = client.request(req);

    response.await.unwrap()
}

async fn handle(
    context: AppContext,
    _addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let headers = req.headers().clone();

    let r = st_req(
        &method,
        &path,
        &headers,
        req.into_body(),
        context.client,
        context.ratelimiter,
    )
    .await;
    // Ok(Response::new(format!("{method} {path}\n").into()));

    Ok(r)
    // Ok(Response::new(Body::from("Hello World")))
}

#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let context = AppContext {
        client: Client::builder().build::<_, hyper::Body>(https),
        ratelimiter: Arc::new(
            Ratelimiter::builder(10, Duration::from_secs(10))
                .max_tokens(10)
                .build()
                .unwrap(),
        ),
    };

    // A `MakeService` that produces a `Service` to handle each connection.
    let make_service = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();

        // You can grab the address of the incoming connection like so.
        let addr = conn.remote_addr();

        // Create a `Service` for responding to the request.
        let service = service_fn(move |req| handle(context.clone(), addr, req));

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(service) }
    });

    // Run the server like above...
    let addr = SocketAddr::from(([0, 0, 0, 0], 8042));

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
