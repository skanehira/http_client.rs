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

    pub fn build(&mut self) -> Vec<u8> {
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

        let mut body = body.join("\r\n").as_bytes().to_vec();
        body.append(&mut "\r\n".as_bytes().to_vec());
        if let Some(data) = &self.body {
            body.append(&mut data.to_vec());
        }
        body.append(&mut "\r\n".as_bytes().to_vec());
        body
    }
}
