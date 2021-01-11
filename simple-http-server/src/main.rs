extern crate data_encoding;
extern crate rand;
extern crate ring;

use data_encoding::HEXUPPER;
use rand::seq::SliceRandom;
use ring::digest;

use std::{
    error,
    io::{self, prelude::*},
    net, str, thread,
};

const BASE_STR: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789_=!?#$";

fn gen_request_id(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect(),
    )
    .unwrap()
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let listener = net::TcpListener::bind("0.0.0.0:1234").expect("Error. failed to bind.");
    for streams in listener.incoming() {
        println!("hoge");
        match streams {
            Err(e) => {
                eprintln!("error: {}", e)
            }
            Ok(stream) => {
                thread::spawn(move || {
                    handler(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
    Ok(())
}

fn handler(mut stream: net::TcpStream) -> Result<(), Box<dyn error::Error>> {
    let request_id = gen_request_id(12);
    println!(
        "incoming connection from {}, request_id: {}",
        stream.peer_addr()?,
        request_id
    );
    let reader = io::BufReader::new(&stream);
    let mut request = String::from("");
    for l in reader.lines().map(|l| l.unwrap()) {
        if l.len() == 0 {
            break;
        }
        request = format!("{}{}\\n", request, l);
        println!(
            " in({request_id}): {line}",
            request_id = request_id,
            line = l
        );
    }
    let hash = HEXUPPER.encode(digest::digest(&digest::SHA256, request.as_bytes()).as_ref());

    let response = format!(
        "HTTP/1.1 200 OK\n\
        Content-Type: application/json\n\
        \n\
        {{\"request_id\": \"{request_id}\",\"request\": \"{request}\",\"hash\": \"{hash}\"}}\n",
        hash = hash,
        request = request,
        request_id = request_id
    );
    for l in response.lines() {
        println!("out({}): {}", request_id, l);
    }
    stream.write_all(response.as_bytes())?;
    return Ok(());
}
