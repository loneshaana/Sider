use core::fmt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

use crate::{
    request::Request,
    resp::bytes_to_resp,
    server_result::{ServerError, ServerMessage, ServerValue},
};

#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}

#[derive(Debug)]
pub enum ConnectionError {
    ServerError(ServerError),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::ServerError(e) => {
                write!(f, "{}", format!("Server error:{}", e))
            }
        }
    }
}

pub async fn handle_connection(
    mut stream: TcpStream,
    server_sender: mpsc::Sender<ConnectionMessage>,
) {
    let mut buffer = [0; 512];
    let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

    loop {
        select! {
            result = stream.read(&mut buffer) => {
                match result {
                    Ok(size) if size != 0 => {
                        let mut index = 0;

                        let resp = match bytes_to_resp(&buffer[..size].to_vec(), &mut index) {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("Error {}", e);
                                return;
                            }
                        };
                        eprintln!("resp {:?}", resp);
                        let request = Request {
                            value: resp,
                            sender: connection_sender.clone()
                        };

                        let connection_message = ConnectionMessage::Request(request);
                        match server_sender.send(connection_message).await {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("Error sending request: {}", e);
                                return;
                            }
                        }
                    }
                    Ok(_) => {
                        eprintln!("Connection Closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("err ={}", e);
                        break;
                    }
                }
            }
            Some(response) = connection_receiver.recv() => {
                let _ = match response {
                    ServerMessage::Data(ServerValue::RESP(v)) => stream.write_all(v.to_string().as_bytes()).await,
                    ServerMessage::Error(e) => {
                        eprintln!("Error: {}", ConnectionError::ServerError(e));
                        return;
                    }
                };
            }
        }
    }
}

pub async fn run_listner(host: String, port: u16, server_sender: mpsc::Sender<ConnectionMessage>) {
    let listner = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    loop {
        tokio::select! {
            connection = listner.accept() => {
                match connection {
                    Ok((stream,_)) => {
                        tokio::spawn(handle_connection(stream, server_sender.clone()));
                    }
                    Err(e) =>{
                        eprintln!("Error: {}",e);
                        continue;
                    }
                }
            }
        }
    }
}
