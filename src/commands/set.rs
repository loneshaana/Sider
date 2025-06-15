use crate::{
    request::Request,
    resp::RESP,
    server::Server,
    server_result::{ServerError, ServerValue},
    set::parse_set_arguments,
};

pub async fn command(server: &mut Server, request: &Request, command: &Vec<String>) {
    let storage = match server.storage.as_mut() {
        Some(storage) => storage,
        None => {
            request.error(ServerError::StorageNotInitialized).await;
            return;
        }
    };

    if command.len() < 3 {
        request
            .error(ServerError::CommandSyntaxError(command.join("")))
            .await;
        return;
    }
    let key = command[1].clone();
    let value = command[2].clone();
    let args = match parse_set_arguments(&command[3..].to_vec()) {
        Ok(args) => args,
        Err(_) => {
            request
                .error(ServerError::CommandSyntaxError(command.join(" ")))
                .await;
            return;
        }
    };

    if let Err(_) = storage.set(key, value, args) {
        request
            .error(ServerError::CommandInternalError(command.join(" ")))
            .await;
        return;
    }
    request
        .data(ServerValue::RESP(RESP::SimpleString("OK".to_string())))
        .await;
}
