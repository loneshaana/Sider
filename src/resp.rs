use std::fmt::{self};

use crate::resp_result::{RESPError, RESPLength, RESPResult};

#[derive(Debug, PartialEq)]
pub enum RESP {
    Array(Vec<RESP>),
    SimpleString(String),
    BulkString(String),
    Null,
}

impl fmt::Display for RESP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            Self::Array(data) => {
                let mut output = String::from("*");
                output.push_str(format!("{}\r\n", data.len()).as_str());
                for elem in data.iter() {
                    output.push_str(elem.to_string().as_str());
                }
                output
            }
            Self::SimpleString(data) => format!("+{}\r\n", data),
            Self::BulkString(data) => format!("${}\r\n{}\r\n", data.len(), data),
            Self::Null => String::from("$-1\r\n"),
        };
        write!(f, "{}", data)
    }
}

fn binary_extract_line(buffer: &[u8], index: &mut usize) -> RESPResult<Vec<u8>> {
    let mut output = Vec::new();

    // we try to read after the end of the buffer
    if *index >= buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }

    // if there is not enough space for \r\n
    // the buffer is definitely invalid
    if buffer.len() - *index - 1 < 2 {
        *index = buffer.len();
        return Err(RESPError::OutOfBounds(*index));
    }
    let mut previous_elem: u8 = buffer[*index].clone();
    let mut separator_found: bool = false;
    let mut final_index: usize = *index;

    // Scan the whole buffer looking for \r\n
    for &elem in buffer[*index..].iter() {
        final_index += 1;
        if elem == b'\n' && previous_elem == b'\r' {
            separator_found = true;
            break;
        }
        previous_elem = elem.clone();
    }
    // If the previous element is not \n
    // we are out of bounds
    if !separator_found {
        *index = final_index;
        return Err(RESPError::OutOfBounds(*index));
    }

    // Copy the bytes from the buffer to the output vector
    output.extend_from_slice(&buffer[*index..final_index - 2]);
    *index = final_index;
    Ok(output)
}

fn binary_extract_line_as_string(buffer: &[u8], index: &mut usize) -> RESPResult<String> {
    let line = binary_extract_line(buffer, index)?;
    let s = String::from_utf8(line);
    Ok(s?)
}

fn binary_extract_bytes(buffer: &[u8], index: &mut usize, length: usize) -> RESPResult<Vec<u8>> {
    let mut output = Vec::new();

    // Check if we allowed to read length bytes
    if *index + length >= buffer.len() {
        return Err(RESPError::OutOfBounds(buffer.len()));
    }

    // else we can read it
    output.extend_from_slice(&buffer[*index..*index + length]);

    // update the index
    *index += length;

    return Ok(output);
}

pub fn resp_remove_type(value: char, buffer: &[u8], index: &mut usize) -> RESPResult<()> {
    if buffer[*index] != value as u8 {
        return Err(RESPError::WrongType);
    }
    *index += 1;
    Ok(())
}

// $5\r\nhello\r\n
pub fn resp_extract_length(buffer: &[u8], index: &mut usize) -> RESPResult<RESPLength> {
    let line = binary_extract_line_as_string(buffer, index)?;
    let length: RESPLength = line.parse()?;
    return Ok(length);
}

// Parse a simple string in the form `+VALUE\r\n`
fn parse_simple_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('+', buffer, index)?;

    let line: String = binary_extract_line_as_string(buffer, index)?;
    Ok(RESP::SimpleString(line))
}

// $5\r\nhello\r\n
fn parse_bulk_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('$', buffer, index)?;

    let length = resp_extract_length(buffer, index)?;
    if length == -1 {
        return Ok(RESP::Null);
    }

    if length < -1 {
        return Err(RESPError::IncorrectLength(length));
    }

    let bytes = binary_extract_bytes(buffer, index, length as usize)?;
    let data = String::from_utf8(bytes)?;

    //Increment the index to skip \r\n
    *index += 2;
    Ok(RESP::BulkString(data))
}

fn parse_array(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('*', buffer, index)?;

    let length = resp_extract_length(buffer, index)?;

    if length < 0 {
        return Err(RESPError::IncorrectLength(length));
    }

    let mut data = Vec::new();

    for _ in 0..length {
        match parser_router(buffer, index) {
            Some(parse_func) => {
                let array_element: RESP = parse_func(buffer, index)?;
                data.push(array_element);
            }
            None => return Err(RESPError::Unknown),
        }
    }
    Ok(RESP::Array(data))
}

fn parser_router(
    buffer: &[u8],
    index: &mut usize,
) -> Option<fn(&[u8], &mut usize) -> RESPResult<RESP>> {
    match buffer[*index] {
        b'+' => Some(parse_simple_string),
        b'$' => Some(parse_bulk_string),
        b'*' => Some(parse_array),
        _ => None,
    }
}

pub fn bytes_to_resp(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    match parser_router(buffer, index) {
        Some(parse_func) => return parse_func(buffer, index),
        _ => Err(RESPError::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_extract_line_empty_buffer() {
        let buffer = "".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(_index)) => {
                assert_eq!(_index, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_single_character() {
        let buffer = "0".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(_index)) => {
                assert_eq!(_index, 1);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_index_too_advanced() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 1;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(_index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_no_separator() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line_half_separator() {
        let buffer = "OK\r".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line_incorrect_separator() {
        let buffer = "OK\n".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_line(buffer, &mut index).unwrap();
        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_extract_line_as_string() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_line_as_string(buffer, &mut index).unwrap();
        assert_eq!(output, String::from("OK"));
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_remove_type() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        resp_remove_type('+', buffer, &mut index).unwrap();
        assert_eq!(index, 1);
    }

    #[test]
    fn test_binary_remove_type_error() {
        let buffer = "*OK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = resp_remove_type('+', buffer, &mut index).unwrap_err();
        assert_eq!(index, 0);
        assert_eq!(error, RESPError::WrongType);
    }

    #[test]
    fn test_parse_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_simple_string(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }

    #[test]
    fn test_bytes_to_resp_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }
    #[test]
    fn test_bytes_to_resp_unknown() {
        let buffer = "?OK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = bytes_to_resp(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::Unknown);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_extract_bytes() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_bytes(buffer, &mut index, 6).unwrap();
        assert_eq!(output, "SOMEBY".as_bytes().to_vec());
        assert_eq!(index, 6);
    }
    #[test]
    fn test_binary_extract_bytes_out_of_bounds() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;
        let error = binary_extract_bytes(buffer, &mut index, 10).unwrap_err();
        assert_eq!(error, RESPError::OutOfBounds(9));
        assert_eq!(index, 0);
    }

    #[test]
    fn test_bytes_to_resp_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("OK")));
        assert_eq!(index, 8);
    }

    #[test]
    fn test_parse_array() {
        let buffer = "*2\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_array(buffer, &mut index).unwrap();
        assert_eq!(
            output,
            RESP::Array(vec![
                RESP::SimpleString(String::from("OK")),
                RESP::BulkString(String::from("VALUE"))
            ])
        );
        assert_eq!(index, 20);
    }

    #[test]
    fn test_parse_array_invalid_length() {
        let buffer = "*-1\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_array(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::IncorrectLength(-1));
        assert_eq!(index, 5);
    }

    #[test]
    fn test_bytes_to_resp_array() {
        let buffer = "*2\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(
            output,
            RESP::Array(vec![
                RESP::SimpleString(String::from("OK")),
                RESP::BulkString(String::from("VALUE"))
            ])
        );
        assert_eq!(index, 20);
    }
}
