use crate::header::*;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u32,
    pub header: HttpHeader,
    pub body: Option<Vec<u8>>,
}
