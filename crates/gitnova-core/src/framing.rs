use std::io::{self, BufRead, Write};

const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

pub fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut content_length = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return if saw_header {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF while reading frame headers",
                ))
            } else {
                Ok(None)
            };
        }
        saw_header = true;

        if line == "\r\n" || line == "\n" {
            break;
        }

        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "malformed frame header"))?;
        if name.eq_ignore_ascii_case("Content-Length") {
            if content_length.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate Content-Length",
                ));
            }
            let length = value.trim().parse::<usize>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
            })?;
            if length > MAX_MESSAGE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "message exceeds size limit",
                ));
            }
            content_length = Some(length);
        }
    }

    let content_length = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

pub fn write_frame(writer: &mut impl Write, body: &[u8]) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(body)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn reads_multiple_frames_without_delimiters_in_body() {
        let input = b"Content-Length: 7\r\n\r\n{\"x\":1}Content-Length: 2\r\n\r\n{}";
        let mut reader = BufReader::new(Cursor::new(input));
        assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"{\"x\":1}");
        assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"{}");
        assert!(read_frame(&mut reader).unwrap().is_none());
    }

    #[test]
    fn rejects_missing_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"X-Test: true\r\n\r\n"));
        assert_eq!(
            read_frame(&mut reader).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn rejects_duplicate_content_length() {
        let input = b"Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}";
        let mut reader = BufReader::new(Cursor::new(input));
        assert_eq!(
            read_frame(&mut reader).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn writes_canonical_header() {
        let mut output = Vec::new();
        write_frame(&mut output, b"{}").unwrap();
        assert_eq!(output, b"Content-Length: 2\r\n\r\n{}");
    }
}
