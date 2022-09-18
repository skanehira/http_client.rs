use std::collections::BTreeMap;
use std::fmt::Display;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct HttpParams(BTreeMap<String, String>);

impl Display for HttpParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::<String>::new();
        for (k, v) in self.0.iter() {
            buf.push(format!("{}={}", k, v));
        }
        write!(f, "{}", buf.join("&"))
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for HttpParams {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut p = Self::new();
        for (k, v) in iter {
            p.add(k, v);
        }
        p
    }
}

impl HttpParams {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
    fn add(&mut self, key: &str, value: &str) {
        self.0.insert(key.into(), value.into());
    }
}
