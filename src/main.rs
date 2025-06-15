use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

use crate::{
    connection::{ConnectionError, ConnectionMessage},
    request::Request,
    resp::bytes_to_resp,
    server::{run_server, Server},
    server_result::ServerMessage,
    storage::Storage,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let storage = Storage::new();
    let mut server = Server::with_new(storage);
    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);
    tokio::spawn(run_server(server, server_receiver));

    let mut interval_timer = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, server_sender.clone()));
                    }
                    Err(e) => {
                        eprintln!("Error {}", e);
                        continue;
                    }
                }
            }
        }
    }
}

// takes lock on the storage and runs the expire_keys function
async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = storage.lock().unwrap();
    guard.expire_keys();
}

async fn handle_connection(mut stream: TcpStream, server_sender: mpsc::Sender<ConnectionMessage>) {
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
                    ServerMessage::Data(v) => stream.write_all(v.to_string().as_bytes()).await,
                    ServerMessage::Error(e) => {
                        eprintln!("Error: {}", ConnectionError::ServerError(e));
                        return;
                    }
                };
            }
        }
    }
}

mod connection;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;
/*
Handling concurrent connections we have
1. multithreading
2. multiprocessing
3. Asynchronous programming
*/
