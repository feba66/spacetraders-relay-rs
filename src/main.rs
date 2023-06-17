use reqwest::{
    blocking::{Client, Response},
    header::{AUTHORIZATION, CONTENT_TYPE},
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
fn req(method: String, path: String, client: &Client, auth: String) -> Response {
    if auth != "".to_owned() {
        return client
            .request(
                Method::from_str(&method).unwrap(),
                "https://api.spacetraders.io/v2/".to_owned() + &path,
            )
            .header(AUTHORIZATION, auth)
            .send()
            .unwrap();
    } else {
        return client
            .request(
                Method::from_str(&method).unwrap(),
                "https://api.spacetraders.io/v2/".to_owned() + &path,
            )
            .send()
            .unwrap();
    };
}
fn handle_connection(mut stream: TcpStream, client: &Client) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request.to_owned());
    let req_line = &http_request[0].to_owned();
    let split: Vec<&str> = req_line.split(" ").collect();
    let method = split[0];
    let path = split[1];
    let auth = false;
    let mut bearer = "".to_owned();
    for line in http_request {
        let l = line.to_owned();
        let s: Vec<&str> = l.split(": ").collect();
        println!("{}", s[0]);
        if s[0] == "Authorization" {
            bearer = s[1].to_owned();
        }
    }

    let now = Instant::now();
    let r = req(
        method.to_owned(),
        path.to_owned(),
        client,
        bearer.to_owned(),
    );

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
