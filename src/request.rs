use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::body::Body;
use crate::header::*;
use crate::method::*;
use crate::params::*;

#[derive(Default)]
pub struct Request {
    pub url: String,
    pub base_url: Option<String>,
    pub method: HttpMethod,
    pub header: Option<HttpHeader>,
    pub params: Option<HttpParams>,
    pub body: Option<Body>,
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
        self.body = Some(Body::new(p));
        self
    }

    pub fn get(url: &str) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Get);
        request
    }

    pub fn post<T: Serialize>(url: &str, body: T) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Post).json(body);
        request
    }

    pub fn put<T: Serialize>(url: &str, body: T) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Put).json(body);
        request
    }

    pub fn delete(url: &str) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Delete);
        request
    }

    pub fn patch<T: Serialize>(url: &str, body: T) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Patch).json(body);
        request
    }

    pub fn head(url: &str) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Head);
        request
    }

    pub fn options(url: &str) -> Self {
        let mut request = Self::new(url.into());
        request.method(HttpMethod::Options);
        request
    }

    pub fn json<T: Serialize>(&mut self, p: T) -> &mut Self {
        let json = serde_json::to_value(p).unwrap();
        self.body = Some(Body::new(json.to_string().as_bytes().to_vec()));
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
            None => "localhost".into(),
        };

        let mut message = vec![
            format!("{} {} HTTP/1.1", self.method, url),
            format!("Host: {}", base_url),
        ];
        if let Some(header) = &self.header {
            message.push(format!("{}", header));
        }
        message.push("".into());

        let mut message = message.join("\r\n").as_bytes().to_vec();
        let mut newline = b"\r\n".to_vec();
        if let Some(data) = &self.body {
            message.append(&mut newline.clone());
            message.append(&mut data.raw());
        }
        message.append(&mut newline);
        message
    }

    pub fn to_string(&self) -> Result<String> {
        let result = self.build();
        String::from_utf8(result).map_err(|x| anyhow!("{}", x))
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[derive(Serialize, Clone)]
    struct Animal {
        name: String,
        age: usize,
    }

    #[test]
    fn request_build() -> Result<()> {
        let req = Request {
            url: "/images/json".into(),
            method: HttpMethod::Get,
            ..Default::default()
        };
        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
        let got = req.to_string()?;
        assert_eq!(want, got);
        Ok(())
    }

    #[test]
    fn request_get() -> Result<()> {
        let req = Request::get("/images/json");
        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
        let got = req.to_string()?;
        assert_eq!(want, got);
        Ok(())
    }

    #[test]
    fn request_with_options() -> Result<()> {
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
        let got = req.to_string()?;
        assert_eq!(want, got);
        Ok(())
    }

    #[test]
    fn with_json() -> Result<()> {
        let animal = Animal {
            name: "gorilla".into(),
            age: 10,
        };

        let mut req = Request::new("/foo".into());
        let req = req.json(animal.clone()).method(HttpMethod::Post);
        let got = req.to_string()?;

        let body = serde_json::to_value(animal)?.to_string();
        let want = [
            "POST /foo HTTP/1.1",
            "Host: localhost",
            "",
            body.as_str(),
            "",
        ]
        .join("\r\n");
        assert_eq!(got, want);
        Ok(())
    }
}
