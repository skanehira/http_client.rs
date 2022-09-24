use anyhow::{anyhow, Result};
use serde::de::Deserialize;

#[derive(Debug, Clone)]
pub struct Body {
    data: Vec<u8>,
}

impl Body {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn raw(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    pub fn json<T: for<'b> Deserialize<'b>>(&self) -> Result<T> {
        serde_json::from_slice(self.data.as_slice()).map_err(|x| anyhow!("{}", x))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn read_as_json() -> Result<()> {
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        struct Gorilla {
            name: String,
            age: usize,
        }
        let body = Body {
            data: r#"{"name": "gorilla", "age": 5}"#.as_bytes().to_vec(),
        };

        let got: Gorilla = body.json()?;
        let want = Gorilla {
            name: "gorilla".into(),
            age: 5,
        };
        assert_eq!(want, got);
        Ok(())
    }
}
