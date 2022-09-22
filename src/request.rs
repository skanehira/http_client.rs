use serde::Serialize;

use crate::header::*;
use crate::method::*;
use crate::params::*;

#[derive(Default)]
pub struct Request {
    url: String,
    base_url: Option<String>,
    method: HttpMethod,
    header: Option<HttpHeader>,
    params: Option<HttpParams>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    pub fn base_url(&mut self, p: String) -> &mut Self {
        self.base_url = Some(p);
        self
    }

    pub fn method(&mut self, p: HttpMethod) -> &mut Self {
        self.method = p;
        self
    }

    pub fn header(&mut self, p: HttpHeader) -> &mut Self {
        self.header = Some(p);
        self
    }

    pub fn params(&mut self, p: HttpParams) -> &mut Self {
        self.params = Some(p);
        self
    }

    pub fn body(&mut self, p: Vec<u8>) -> &mut Self {
        self.body = Some(p);
        self
    }

    pub fn get(url: &str) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Get);
        request
    }

    pub fn json<T: Serialize>(&mut self, p: T) -> &mut Self {
        let json = serde_json::to_value(p).unwrap();
        self.body = Some(json.to_string().as_bytes().to_vec());
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let url = match &self.params {
            Some(params) => {
                format!("{}?{}", self.url, params)
            }
            None => self.url.clone(),
        };

        let base_url = match &self.base_url {
            Some(base_url) => base_url.clone(),
            None => "localhost".to_string(),
        };

        let mut body = vec![
            format!("{} {} HTTP/1.1", self.method, url),
            format!("Host: {}", base_url),
        ];
        if let Some(header) = &self.header {
            body.push(format!("{}", header));
        }
        body.push("".into());

        let mut body = body.join("\r\n").as_bytes().to_vec();
        let crlf = "\r\n".as_bytes().to_vec();
        if let Some(data) = &self.body {
            body.append(&mut crlf.clone());
            body.append(&mut data.to_vec());
        }
        body.append(&mut crlf.clone());
        body
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize)]
    struct Animal {
        name: String,
        age: usize,
    }

    #[test]
    fn request_build() {
        let req = Request {
            url: "/images/json".to_string(),
            method: HttpMethod::Get,
            ..Default::default()
        };
        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
        let got = String::from_utf8(req.build()).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn request_get() {
        let req = Request::get("/images/json");
        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
        let got = String::from_utf8(req.build()).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn request_with_options() {
        let mut req = Request::new("/images/json".into());
        let params: HttpParams = [("name", "nvim"), ("image", "ubuntu")]
            .into_iter()
            .collect();

        let header: HttpHeader = [("bar", "1000"), ("foo", "value")].into_iter().collect();

        req.method(HttpMethod::Post)
            .params(params)
            .header(header)
            .body("test body".as_bytes().to_vec());

        let want = [
            "POST /images/json?image=ubuntu&name=nvim HTTP/1.1",
            "Host: localhost",
            "bar: 1000",
            "foo: value",
            "",
            "test body",
            "",
        ]
        .join("\r\n");
        let got = String::from_utf8(req.build()).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn with_json() {
        let g = Animal {
            name: "gorilla".into(),
            age: 10,
        };

        let mut req = Request::new("/foo".into());
        let got = String::from_utf8(req.json(g).body.clone().unwrap()).unwrap();
        let want = r#"{"age":10,"name":"gorilla"}"#;
        assert_eq!(got, want);
    }
}
