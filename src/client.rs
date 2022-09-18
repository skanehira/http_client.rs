use crate::header::*;
use crate::request::*;
use crate::response::*;
use std::io::{self, BufRead, BufReader, Read};

pub trait ReadWriter: io::Read + io::Write {}

// NOTE: io::Read と io::Write を満たしているすべての T に対して、ReadWriter を実装する
// つまり、これで io::Read と io::Write 両方を実装している構造体などに ReadWriter
// を実装したことになる
impl<T> ReadWriter for T where T: io::Read + io::Write {}

pub struct HttpClient<T: ReadWriter> {
    conn: T,
}

impl<T: ReadWriter> HttpClient<T> {
    pub fn new(conn: T) -> Self {
        HttpClient { conn }
    }

    fn read_response(&mut self) -> Result<Response, String> {
        let mut r = BufReader::new(&mut self.conn);
        let mut buf = Vec::new();

        // read status line
        r.read_until(b'\n', &mut buf).unwrap();
        let status_line = String::from_utf8(buf.clone())
            .map_err(|_| "cannot convert bytes to string".to_string())?;

        let status = status_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| "cannot get status code".to_string())?
            .parse::<u32>()
            .map_err(|_| "cannot parse to number".to_string())?;

        // read headers
        let mut header = HttpHeader::new();
        loop {
            buf.clear();
            let readed = r
                .read_until(b'\n', &mut buf)
                .map_err(|_| "cannot read header".to_string())?;

            if readed == 0 {
                return Err("unexpected endof".to_string());
            }

            let mut line = String::from_utf8(buf.clone())
                .map_err(|_| "cannot coonvert bytes to string".to_string())?;
            if line == "\r\n" {
                break;
            }
            line = line.trim().to_string();

            let mut cols = line.split(": ");
            let key = cols
                .next()
                .ok_or_else(|| "invalid header key".to_string())?
                .to_lowercase();
            let key = key.as_str();
            let val = cols
                .next()
                .ok_or_else(|| "invalid header value".to_string())?;

            header.add(key, val);
        }

        match status {
            204 | 304 => {
                let resp = Response {
                    status,
                    header,
                    body: None,
                };
                return Ok(resp);
            }
            _ => {}
        }

        let tf = header.get("transfer-encoding");
        let cl = header.get("content-length");

        if tf.is_none() && cl.is_none() {
            return Err("missing transfer-encoding or content-length".into());
        }

        let is_chunked = tf.map(|x| *x == "chunked").unwrap_or(false);

        let mut body = Vec::new();
        if is_chunked {
            // read body
            loop {
                buf.clear();
                let readed = r.read_until(b'\n', &mut buf).unwrap();
                if readed == 0 {
                    break;
                }

                let line = String::from_utf8(buf.clone())
                    .map_err(|_| "cannot coonvert bytes to string".to_string())?;
                let chunk_size = i64::from_str_radix(line.trim(), 16)
                    .map_err(|err| format!("cannot read chunk length: {}: {}", line, err))?;

                if chunk_size == 0 {
                    let _ = r.read_until(b'\n', &mut buf);
                    break;
                }

                let mut chunk = vec![0u8; chunk_size as usize];
                r.read_exact(&mut chunk).unwrap();
                body.append(&mut chunk);

                // consume \r\n
                let _ = r.read_until(b'\n', &mut buf);
            }
        } else {
            let value = header.get("content-length");
            if value.is_none() {
                return Err("not found content-length".into());
            }
            let value = value.unwrap().parse::<isize>();

            match value {
                Ok(size) => {
                    let mut buf = vec![0u8; size.to_owned() as usize];
                    r.read_exact(&mut buf).unwrap();
                    body = buf;
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            };
        }

        let resp = Response {
            status,
            header,
            body: Some(body),
        };
        Ok(resp)
    }

    fn execute_request(&mut self, req: &mut Request) -> Result<Response, String> {
        let body = req.build();
        self.conn.write_all(&body).unwrap();
        self.read_response()
    }
}
