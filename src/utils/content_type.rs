use std::fmt;

use axum::http::header;
use headers::{Error, Header, HeaderName, HeaderValue};
use mediatype::{
    names::{APPLICATION, JSON, XML},
    MediaTypeBuf,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ContentType(pub MediaTypeBuf);

impl ContentType {
    #[inline]
    pub fn json() -> ContentType {
        ContentType(MediaTypeBuf::new(APPLICATION, JSON))
    }

    #[inline]
    pub fn xml() -> ContentType {
        ContentType(MediaTypeBuf::new(APPLICATION, XML))
    }
}

impl Header for ContentType {
    fn name() -> &'static HeaderName {
        &header::CONTENT_TYPE
    }

    fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(values: &mut I) -> Result<Self, Error> {
        values
            .next()
            .and_then(|v| v.to_str().ok()?.parse().ok())
            .map(ContentType)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = self
            .0
            .as_ref()
            .parse()
            .expect("Mime is always a valid HeaderValue");
        values.extend(::std::iter::once(value));
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::str::FromStr for ContentType {
    type Err = Error;

    fn from_str(s: &str) -> Result<ContentType, Self::Err> {
        s.parse::<MediaTypeBuf>()
            .map(Self)
            .map_err(|_| Error::invalid())
    }
}
