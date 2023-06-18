use hyper::body::HttpBody as _;
use hyper::client::connect::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body, Body, Client, HeaderMap, Method, Request, Response, Server, Uri};
use hyper_tls::HttpsConnector;
use std::convert::Infallible;
use std::net::SocketAddr;
const GAME_SERVER_URL: &str = "https://api.spacetraders.io/v2/";

#[derive(Clone)]
struct AppContext {
    client: Client<HttpsConnector<HttpConnector>>,
}

async fn st_req(
    method: &Method,
    path: &String,
    headers: &HeaderMap,
    body: Body,
    client: Client<HttpsConnector<HttpConnector>>,
) -> Response<Body> {
    // let bbtes = hyper::body::to_bytes(b);
    let mut r = Request::builder()
        .method(method)
        .uri(GAME_SERVER_URL.to_owned() + &path);
    if headers.contains_key(AUTHORIZATION) {
        r = r.header(AUTHORIZATION, headers.get(AUTHORIZATION).unwrap());
    }
    if headers.contains_key(CONTENT_TYPE) {
        r = r.header(CONTENT_TYPE, headers.get(CONTENT_TYPE).unwrap());
    }
    let req = r.body(body).unwrap();
    let response = client.request(req);

    return response.await.unwrap();
}

async fn handle(
    context: AppContext,
    _addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {

    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let headers = req.headers().clone();

    let r = st_req(&method, &path, &headers, req.into_body(), context.client).await;
    // Ok(Response::new(format!("{method} {path}\n").into()));

    Ok(r)
    // Ok(Response::new(Body::from("Hello World")))
}

#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let context = AppContext {
        client: Client::builder().build::<_, hyper::Body>(https),
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
