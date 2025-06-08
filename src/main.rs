use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{resp::bytes_to_resp, server::process_request, storage::Storage};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let storage = Arc::new(Mutex::new(Storage::new()));
    let mut interval_timer = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, storage.clone()));
                    }
                    Err(e) => {
                        eprintln!("Error {}", e);
                        continue;
                    }
                }
            }
            _ = interval_timer.tick() =>{
                tokio::spawn(expire_keys(storage.clone()));
            }
        }
    }
}

// takes lock on the storage and runs the expire_keys function
async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = storage.lock().unwrap();
    guard.expire_keys();
}

async fn handle_connection(mut stream: TcpStream, storage: Arc<Mutex<Storage>>) {
    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer).await {
            Ok(size) if size != 0 => {
                let mut index = 0;
                let request = match bytes_to_resp(&buffer[..size].to_vec(), &mut index) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error {}", e);
                        return;
                    }
                };
                eprint!("request {:?} \r\n", request);
                let response = match process_request(request, storage.clone()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprint!("Error parsing command :{} \r\n", e);
                        return;
                    }
                };
                if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
                    eprintln!("Error writing to socket:{}", e);
                }
            }
            Ok(_) => {
                println!("Connection Closed");
                break;
            }
            Err(e) => {
                println!("err ={}", e);
                break;
            }
        }
    }
}
mod resp;
mod resp_result;
mod server;
mod storage;
mod storage_result;
/*
Handling concurrent connections we have
1. multithreading
2. multiprocessing
3. Asynchronous programming
*/
