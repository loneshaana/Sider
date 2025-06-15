use std::time::Duration;

use tokio::sync::mpsc;

use crate::{
    connection::ConnectionMessage,
    request::Request,
    resp::RESP,
    server_result::{ServerError, ServerValue},
    storage::Storage,
};

pub struct Server {
    pub storage: Option<Storage>,
}

impl Server {
    pub fn new() -> Self {
        Self { storage: None }
    }

    pub fn with_new(storage: Storage) -> Self {
        Self {
            storage: Some(storage),
        }
    }

    pub fn set_storage(&mut self, storage: Storage) {
        self.storage = Some(storage);
    }

    pub fn expire_keys(&mut self) {
        let storage = match self.storage.as_mut() {
            Some(storage) => storage,
            None => return,
        };
        storage.expire_keys();
    }
}

pub async fn run_server(mut server: Server, mut crx: mpsc::Receiver<ConnectionMessage>) {
    let mut internal_timer = tokio::time::interval(Duration::from_millis(10));

    loop {
        tokio::select! {
            Some(message) = crx.recv() => {
                match message {
                    ConnectionMessage::Request(request) => {
                        process_request(request, &mut server).await;
                    }
                }
            }
            _ = internal_timer.tick() =>{
                server.expire_keys();
            }
        }
    }
}

pub async fn process_request(request: Request, server: &mut Server) {
    let elements = match &request.value {
        RESP::Array(v) => v,
        _ => {
            request.error(ServerError::IncorrectData).await;
            return;
        }
    };
    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.to_owned()),
            _ => {
                request.error(ServerError::IncorrectData).await;
                return;
            }
        }
    }

    let storage = match server.storage.as_mut() {
        Some(storage) => storage,
        None => {
            request.error(ServerError::StorageNotInitialized).await;
            return;
        }
    };

    let response = storage.process_command(&command);
    match response {
        Ok(v) => {
            request
                .sender
                .send(crate::server_result::ServerMessage::Data(
                    ServerValue::RESP(v),
                ))
                .await
                .unwrap();
        }
        Err(e) => (),
    };
}

#[cfg(test)]
mod tests {
    use crate::server_result::ServerMessage;
    use tokio::sync::mpsc;

    use super::*;
    #[tokio::test]
    async fn test_process_request_ping() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
        let request = Request {
            value: RESP::Array(vec![RESP::BulkString(String::from("PING"))]),
            sender: connection_sender,
        };
        let mut server = Server::with_new(Storage::new());
        process_request(request, &mut server).await;
        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::SimpleString(String::from("PONG"))))
        );
    }
    #[tokio::test]
    async fn test_process_request_not_array() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
        let request = Request {
            value: RESP::BulkString(String::from("PING")),
            sender: connection_sender,
        };
        let mut server = Server::with_new(Storage::new());
        process_request(request, &mut server).await;
        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Error(ServerError::IncorrectData)
        );
    }

    #[tokio::test]
    async fn test_process_request_not_bulkstrings() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
        let request = Request {
            value: RESP::Array(vec![RESP::SimpleString(String::from("PING"))]),
            sender: connection_sender,
        };
        let mut server = Server::with_new(Storage::new());
        process_request(request, &mut server).await;
        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Error(ServerError::IncorrectData)
        );
    }

    #[tokio::test]
    async fn test_process_request_echo() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
        let request = Request {
            value: RESP::Array(vec![
                RESP::BulkString(String::from("ECHO")),
                RESP::BulkString(String::from("42")),
            ]),
            sender: connection_sender,
        };
        let mut server = Server::with_new(Storage::new());
        process_request(request, &mut server).await;
        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::BulkString(String::from("42"))))
        );
    }

    #[test]
    fn test_create_new() {
        let server: Server = Server::new();
        match server.storage {
            Some(_) => panic!(),
            None => (),
        };
    }
    #[test]
    fn test_set_storage() {
        let storage = Storage::new();
        let mut server: Server = Server::new();
        server.set_storage(storage);
        match server.storage {
            Some(_) => (),
            None => panic!(),
        };
    }
}
