use reqwest::blocking::Client;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
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
fn handle_connection(mut stream: TcpStream, client: &Client) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);
    let now = Instant::now();
    let r = client
        .get("https://api.spacetraders.io/v2/")
        .send()
        .unwrap();
    println!("R: {:#?}", r.headers());

    let elapsed_time = now.elapsed();
    println!("Request took {} seconds.", elapsed_time.as_secs_f32());
    let response = format!("HTTP/1.1 {} OK\r\n\r\n", r.status().as_str());

    stream.write_all(response.as_bytes()).unwrap();
}
