use actix_web::{App, get, HttpServer, post};
use actix_web::web::{Data, Json, JsonConfig};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::process::exit;
use std::sync::Mutex;
use structopt::StructOpt;
use ureq::{json, post};

struct AppState {
  client_messages: Mutex<Vec<String>>,
  peer_messages: Mutex<Vec<String>>,
  peers: Vec<u16>
}

#[derive(StructOpt)]
struct Args {
  #[structopt(short, long)]
  port: u16
}

#[derive(Deserialize)]
struct Message {
  msg: String
}

#[post("/app/msg")]
async fn client_handler(data: Data<AppState>, request: Json<Message>) -> String {
  match (& data.client_messages).lock() {
    Ok(mut messages) => {
      let payload = request.msg.clone();
      messages.push(payload.clone());
      let peers = & data.peers;
      let _: Vec<()> =
        peers
          .iter()
          .map(|port| match post(& format!("http://localhost:{}/app/relay", port)).send_json(json!({ "msg": & payload.clone() })) {
            Ok(_) => (),
            Err(_) => println!("Error relaying message {} to peers", request.msg.clone())
          }).collect();
    },
    Err(_) => println!("Error acquiring lock or data for client received message upon request {}", request.msg.clone())
  };
  request.msg.clone()
}

#[post("/app/relay")]
async fn peer_handler(data: Data<AppState>, request: Json<Message>) -> String {
  match (& data.peer_messages).lock() {
    Ok(mut messages) => messages.push(request.msg.clone()),
    Err(_) => println!("Error acquiring lock or data for peer received message upon request {}", request.msg.clone())
  };
  request.msg.clone()
}

#[get("/app/digest")]
async fn digest_handler(data: Data<AppState>) -> String {
  match ((& data.client_messages).lock(), (& data.peer_messages).lock()) {
    (Ok(client_messages), Ok(peer_messages)) => {
      let message_count = client_messages.len() + peer_messages.len();
      let mut raw_messages: Vec<String> = vec![];
      let _ = client_messages.iter().map(|message| raw_messages.push(message.to_string())).collect::<Vec<()>>();
      let _ = peer_messages.iter().map(|message| raw_messages.push(message.to_string())).collect::<Vec<()>>();
      raw_messages.sort();
      let mut digest = Sha256::new();
      digest.update(raw_messages.join(""));
      format!("{{ \"digest\": \"{:X}\", \"count\": {} }}", digest.finalize(), message_count)
    },
    (Ok(_), Err(_)) | (Err(_), Ok(_)) | (Err(_), Err(_)) => "{{ \"error\": \"Error retrieving digest\" }}".to_string()
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  println!("Server starting");
  let arguments = Args::from_args();
  let port = match arguments.port {
    9001 | 9002 | 9003 => arguments.port,
    _ => {
      println!("Only ports 9001, 9002, and 9003 are expected.  Port requested: {}", arguments.port);
      exit(1);
    }
  };
  let state = Data::new(AppState {
    client_messages: Mutex::new(vec![]),
    peer_messages: Mutex::new(vec![]),
    peers: vec![9001, 9002, 9003].into_iter().filter(|p| * p != port).collect::<Vec<u16>>()
  });
  HttpServer::new(move || {
    App::new()
      .app_data(state.clone())
      .app_data(JsonConfig::default())
      .service(client_handler)
      .service(peer_handler)
      .service(digest_handler)
  })
  .bind(format!(
    "127.0.0.1:{}",
    port
  ))?
  .run()
  .await
}