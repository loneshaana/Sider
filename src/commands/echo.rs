use crate::server_result::ServerValue;
use crate::{request::Request, resp::RESP, server::Server};

pub async fn command(_server: &Server, request: &Request, command: &Vec<String>) {
    request
        .data(ServerValue::RESP(RESP::BulkString(command[1].clone())))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_result::ServerMessage;
    use tokio::sync::mpsc;
    #[tokio::test]
    async fn test_command() {
        let cmd = vec![String::from("echo"), String::from("hey")];
        let server = Server::new();
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
        let request = Request {
            value: RESP::Null,
            sender: connection_sender,
        };
        command(&server, &request, &cmd).await;
        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::BulkString(String::from("hey"))))
        );
    }
}
