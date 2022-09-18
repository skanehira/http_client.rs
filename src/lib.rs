mod client;
mod header;
mod method;
mod params;
mod request;
mod response;

//#[cfg(test)]
//mod test {
//    use super::*;
//
//    #[test]
//    fn request_build() {
//        let mut req = Request {
//            url: "/images/json".to_string(),
//            method: HttpMethod::Get,
//            ..Default::default()
//        };
//        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
//        let got = String::from_utf8(req.build()).unwrap();
//        assert_eq!(want, got);
//    }
//
//    #[test]
//    fn request_get() {
//        let mut req = Request::get("/images/json");
//        let want = ["GET /images/json HTTP/1.1", "Host: localhost", "", ""].join("\r\n");
//        let got = String::from_utf8(req.build()).unwrap();
//        assert_eq!(want, got);
//    }
//
//    #[test]
//    fn request_with_options() {
//        let mut req = Request::new("/images/json".into());
//        let params: HttpParams = [("name", "nvim"), ("image", "ubuntu")]
//            .into_iter()
//            .collect();
//
//        let header: HttpHeader = [("bar", "1000"), ("foo", "value")].into_iter().collect();
//
//        req.method(HttpMethod::Get).params(params).header(header);
//
//        let want = [
//            "GET /images/json?image=ubuntu&name=nvim HTTP/1.1",
//            "Host: localhost",
//            "bar: 1000",
//            "foo: value",
//            "",
//            "",
//        ]
//        .join("\r\n");
//        let got = String::from_utf8(req.build()).unwrap();
//        assert_eq!(want, got);
//    }
//}
