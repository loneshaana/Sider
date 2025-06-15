use tokio::sync::mpsc;

use crate::{
    connection::{run_listner, ConnectionMessage},
    server::{run_server, Server},
    storage::Storage,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);
    let mut storage = Storage::new();
    storage.set_active_expiry(true);

    let server = Server::with_new(storage);
    tokio::spawn(run_server(server, server_receiver));

    run_listner("127.0.0.1".to_string(), 6379, server_sender).await;
    Ok(())
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
