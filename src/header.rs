use std::collections::BTreeMap;
use std::fmt::Display;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct HttpHeader(BTreeMap<String, String>);

impl Display for HttpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut h = Vec::new();
        for (k, v) in self.0.iter() {
            h.push(format!("{}: {}", k, v));
        }
        write!(f, "{}", h.join("\r\n"),)
    }
}

impl HttpHeader {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn add(&mut self, key: &str, value: &str) {
        self.0.insert(key.into(), value.into());
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }
}

impl Default for HttpHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for HttpHeader {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut p = Self::new();
        for (k, v) in iter {
            p.add(k, v);
        }
        p
    }
}
