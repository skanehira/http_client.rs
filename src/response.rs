use crate::body::Body;
use crate::header::*;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u32,
    pub header: HttpHeader,
    pub body: Option<Body>,
}
