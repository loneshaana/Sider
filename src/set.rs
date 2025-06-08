use crate::storage_result::{StorageError, StorageResult};

#[derive(Debug, PartialEq)]
pub enum KeyExistence {
    NX, //set if not exists
    XX, // set if exists
}

#[derive(Debug, PartialEq)]
pub enum KeyExipry {
    EX(u64), // expiry in seconds
    PX(u64), // expiry in milliseconds
}

#[derive(Debug, PartialEq)]
pub struct SetArgs {
    pub expiry: Option<KeyExipry>,
    pub existence: Option<KeyExistence>,
    pub get: bool,
}

impl SetArgs {
    pub fn new() -> Self {
        Self {
            expiry: None,
            existence: None,
            get: false,
        }
    }
}

pub fn parse_set_arguments(arguments: &Vec<String>) -> StorageResult<SetArgs> {
    let mut args = SetArgs::new();
    let mut idx: usize = 0;
    loop {
        if idx >= arguments.len() {
            break;
        }
        match arguments[idx].to_lowercase().as_str() {
            "nx" => {
                if args.existence == Some(KeyExistence::XX) {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }
                args.existence = Some(KeyExistence::NX);
                idx += 1;
            }
            "xx" => {
                if args.existence == Some(KeyExistence::NX) {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }
                args.existence = Some(KeyExistence::XX);
                idx += 1;
            }
            "ex" => {
                if let Some(KeyExipry::PX(_)) = args.expiry {
                    // PX is set , we can't set EX
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }
                if idx + 1 == arguments.len() {
                    // expecting a value for EX
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }
                let value: u64 = arguments[idx + 1]
                    .parse()
                    .map_err(|_| StorageError::CommandSyntaxError(arguments.join(" ")))?;
                args.expiry = Some(KeyExipry::EX(value));
                idx += 2;
            }
            "px" => {
                if let Some(KeyExipry::EX(_)) = args.expiry {
                    // PX is set , we can't set EX
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }
                if idx + 1 == arguments.len() {
                    // expecting a value for EX
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                let value: u64 = arguments[idx + 1]
                    .parse()
                    .map_err(|_| StorageError::CommandSyntaxError(arguments.join(" ")))?;
                args.expiry = Some(KeyExipry::PX(value));
                idx += 2;
            }
            "get" => {
                args.get = true;
                idx += 1;
            }
            _ => {
                return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
            }
        }
    }
    Ok(args)
}
