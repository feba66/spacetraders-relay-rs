use reqwest::{
    blocking::{Client, Response},
    header::CONTENT_TYPE,
    Method,
};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    str::FromStr,
    time::Instant,
};
fn main() {
    let client = Client::builder()
        .user_agent("SpaceTraders-Relay-rs")
        .build()
        .unwrap();
    let listener = TcpListener::bind("0.0.0.0:8042").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &client);
    }
}
fn req(method: String, path: String, client: &Client) -> Response {
    let r = client
        .request(
            Method::from_str(&method).unwrap(),
            "https://api.spacetraders.io/v2/".to_owned() + &path,
        )
        .send()
        .unwrap();
    return r;
}
fn handle_connection(mut stream: TcpStream, client: &Client) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let req_line = &http_request[0];
    let split:Vec<&str> = req_line.split(" ").collect();
    let method = split[0];
    let path = split[1];

    println!("Request: {:#?}", http_request);
    let now = Instant::now();
    let r = req(method.to_owned(), path.to_owned(), client);

    let status = r.status().to_string();
    let clen = r.content_length().unwrap().to_string();
    let content_type = r
        .headers()
        .get(CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let content = r.text().unwrap();
    let response = format!("HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {clen}\r\n\r\n{content}", );

    let elapsed_time = now.elapsed();
    println!("Request took {} seconds.", elapsed_time.as_secs_f32());
    println!("{}", response);
    stream.write_all(response.as_bytes()).unwrap();
}
