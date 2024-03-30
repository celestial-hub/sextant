#![feature(async_closure)]

use anyhow::Context;
use axum::{
  extract::{
    ws::{Message, WebSocket},
    WebSocketUpgrade,
  },
  response::IntoResponse,
  routing::get,
  Router,
};
use celestial_hub_astrolabe::{lexer::Lexer, parser::Parser};

use celestial_hub_sextant::{vm::VM, CommandMessage, ErrorMessage, SocketMessage, VMCommand};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let app = Router::new().route("/ws", get(ws_handler));
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .context("failed to bind to address")?;

  tracing::debug!("listening on {listener}", listener = listener.local_addr()?);

  axum::serve(
    listener,
    app.into_make_service_with_connect_info::<SocketAddr>(),
  )
  .await
  .context("failed to bind server to listener address")?;

  Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
  ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
  let vm = VM::new(Default::default());

  if let Err(e) = vm {
    let socket_error_message = &SocketMessage::Error(ErrorMessage {
      message: e.to_string(),
    });
    let error_message = Message::Text(serde_json::to_string(socket_error_message).unwrap());

    socket.send(error_message).await.unwrap();
    socket.send(Message::Close(None)).await.unwrap();
    return;
  }

  let mut vm: VM = vm.expect("VM to be initialized");
  if let Err(e) = vm.load(get_code()) {
    let socket_error_message = &SocketMessage::Error(ErrorMessage {
      message: e.to_string(),
    });
    let error_message = Message::Text(serde_json::to_string(socket_error_message).unwrap());

    socket.send(error_message).await.unwrap();
    socket.send(Message::Close(None)).await.unwrap();
    return;
  }

  while let Some(Ok(msg)) = socket.recv().await {
    match msg {
      Message::Text(text) => {
        let command = serde_json::from_str::<SocketMessage>(&text);
        tracing::debug!("Received command: {:?}", command);

        if let Err(e) = command {
          tracing::error!("Failed to parse command: {}", e);
          let socket_error_message = &SocketMessage::Error(ErrorMessage {
            message: e.to_string(),
          });
          let error_message = Message::Text(serde_json::to_string(socket_error_message).unwrap());

          socket.send(error_message).await.unwrap();
          socket.send(Message::Close(None)).await.unwrap();
          return;
        }

        match command.unwrap() {
          SocketMessage::Command(CommandMessage { command }) => match command {
            VMCommand::Step => {
              vm.step().unwrap();
              let status_update_message = &SocketMessage::StatusUpdate(vm.get_status());
              let message = Message::Text(serde_json::to_string(status_update_message).unwrap());

              socket.send(message).await.unwrap();
            }
            VMCommand::Run => {
              vm.run().unwrap();
              let status_update_message = &SocketMessage::StatusUpdate(vm.get_status());
              let message = Message::Text(serde_json::to_string(status_update_message).unwrap());

              socket.send(message).await.unwrap();
            }
          },
          SocketMessage::Input(input) => {
            if let Err(e) = vm.handle_input(input) {
              tracing::error!("Failed to handle input: {}", e);
              let socket_error_message = &SocketMessage::Error(ErrorMessage {
                message: e.to_string(),
              });
              let error_message =
                Message::Text(serde_json::to_string(socket_error_message).unwrap());
              socket.send(error_message).await.unwrap();
            }
          }
          _ => { /* Ignore */ }
        }
      }
      Message::Close(close_frame) => {
        if let Some(close_frame) = close_frame {
          tracing::debug!("closing socket: {:?}", close_frame);
        }
        return;
      }
      _ => { /* Ignore */ }
    }
  }
}

fn get_code() -> celestial_hub_astrolabe::ast::Program {
  let source_code = r#"
  .data
prompt: .asciiz "The sum of is: "

	.text
	.global main
main:
  ; Read number 1
  li $v0, 5
  syscall
  move $t0, $v0

  ; Read number 2
  li $v0, 5
  syscall
  move $t1, $v0

  ; Add numbers
	add $t2, $t0, $t1

  ; Print prompt
  li $v0, 4
  la $a0, prompt
  syscall

  ; Print added value
  li $v0, 1
  move $a0, $t2
  syscall

  ; Exit
  li $v0, 0xA
  syscall
    "#;

  Parser::new()
    .parse(Lexer::new(source_code, "test_eval"))
    .expect("parse failed")
}
