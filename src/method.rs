use std::fmt::Display;

pub enum HttpMethod {
    Get,
    Post,
    Update,
    Delete,
    Patch,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        };
        write!(f, "{}", method)
    }
}
