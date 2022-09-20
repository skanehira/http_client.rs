use crate::header::*;
use crate::request::*;
use crate::response::*;
use anyhow::anyhow;
use anyhow::Result;
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

    fn read_response(&mut self) -> Result<Response> {
        let mut r = BufReader::new(&mut self.conn);
        let mut buf = Vec::new();

        // read status line
        r.read_until(b'\n', &mut buf).unwrap();
        let status_line = String::from_utf8(buf.clone())?;

        let status = status_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| anyhow!("cannot get status code"))?
            .parse::<u32>()?;

        // read headers
        let mut header = HttpHeader::new();
        loop {
            buf.clear();
            let readed = r.read_until(b'\n', &mut buf)?;

            if readed == 0 {
                return Err(anyhow!("unexpected endof"));
            }

            let mut line = String::from_utf8(buf.clone())?;
            if line == "\r\n" {
                break;
            }
            line = line.trim().to_string();

            let mut cols = line.split(": ");
            let key = cols
                .next()
                .ok_or_else(|| anyhow!("invalid header key"))?
                .to_lowercase();
            let key = key.as_str();
            let val = cols.next().ok_or_else(|| anyhow!("invalid header value"))?;

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
            return Err(anyhow!("missing transfer-encoding or content-length"));
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
                    .map_err(|_| anyhow!("cannot coonvert bytes to string"))?;
                let chunk_size = i64::from_str_radix(line.trim(), 16)
                    .map_err(|err| anyhow!("cannot read chunk length: {}: {}", line, err))?;

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
                return Err(anyhow!("not found content-length"));
            }
            let value = value.unwrap().parse::<isize>();

            match value {
                Ok(size) => {
                    let mut buf = vec![0u8; size.to_owned() as usize];
                    r.read_exact(&mut buf).unwrap();
                    body = buf;
                }
                Err(e) => {
                    return Err(anyhow!(e.to_string()));
                }
            };
        }

        let resp = Response {
            status,
            header,
            body: Some(Body::new(body)),
        };
        Ok(resp)
    }

    pub fn execute_request(&mut self, req: &Request) -> Result<Response> {
        let body = req.build();
        self.conn.write_all(&body).unwrap();
        self.read_response()
    }
}

#[cfg(test)]
mod test {
    use crate::method::HttpMethod;

    use super::*;
    use anyhow::Result;
    use httptest::{matchers::*, responders::*, Expectation, ServerBuilder};
    use serde_json::json;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};

    #[test]
    fn request_get() -> Result<()> {
        let want_body = r#"{"name": "gorilla", "age": 5}"#;

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let server = ServerBuilder::new().bind_addr(addr).run()?;
        server.expect(
            Expectation::matching(request::method_path("GET", "/hello")).respond_with(
                status_code(200)
                    .append_header("Content-Type", "application/json")
                    .body(want_body),
            ),
        );

        let conn = TcpStream::connect(addr.to_string())?;
        let mut client = HttpClient::new(conn);
        let req = Request::new("/hello".into());
        let resp = client.execute_request(&req)?;
        let body = resp.body.unwrap();

        assert_eq!(body.text()?, want_body);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.header.get("content-type").unwrap(), "application/json");

        Ok(())
    }

    #[test]
    fn request_post() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);

        let want_body = r#"{"name":"gorilla","age":5}"#;
        let server = ServerBuilder::new().bind_addr(addr).run()?;

        server.expect(
            Expectation::matching(all_of![
                request::method("POST"),
                request::path("/hello"),
                request::body(want_body),
            ])
            .respond_with(json_encoded(json!(true))),
        );

        let conn = TcpStream::connect(addr.to_string())?;
        let mut client = HttpClient::new(conn);

        let header: HttpHeader = [
            ("Content-type", "application/json"),
            ("Content-length", want_body.len().to_string().as_str()),
        ]
        .into_iter()
        .collect();

        let mut req = Request::new("/hello".into());
        let req = req
            .body(want_body.clone().as_bytes().to_vec())
            .header(header)
            .method(HttpMethod::Post);
        let resp = client.execute_request(&req)?;
        let body = resp.body.unwrap();
        assert_eq!(body.text().unwrap(), "true");

        Ok(())
    }
}
