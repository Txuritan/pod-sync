use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    RequestExt,
};
use axum_extra::TypedHeader;
use headers_accept::Accept;
use mediatype::{
    names::{APPLICATION, CHARSET, JSON, TEXT, XML},
    values::UTF_8,
    MediaType, MediaTypeBuf,
};

use crate::utils::{
    content_type::ContentType,
    json::{Json, JsonRejection},
    xml::{Xml, XmlRejection},
};

pub const APPLICATION_JSON: MediaType<'static> = MediaType::new(APPLICATION, JSON);
pub const APPLICATION_XML_UTF8: MediaType<'static> =
    MediaType::from_parts(APPLICATION, XML, None, &[(CHARSET, UTF_8)]);
pub const APPLICATION_XML: MediaType<'static> = MediaType::new(APPLICATION, XML);
pub const TEXT_XML_UTF8: MediaType<'static> =
    MediaType::from_parts(TEXT, XML, None, &[(CHARSET, UTF_8)]);
pub const TEXT_XML: MediaType<'static> = MediaType::new(TEXT, XML);

const SUPPORTED_MEDIA_TYPES: &[MediaType<'static>] = &[
    APPLICATION_JSON,
    APPLICATION_XML_UTF8,
    TEXT_XML_UTF8,
    APPLICATION_XML,
    TEXT_XML,
];

fn is_json(typ: &MediaTypeBuf) -> bool {
    typ == APPLICATION_JSON
}

fn is_xml(typ: &MediaTypeBuf) -> bool {
    typ == APPLICATION_XML_UTF8 || typ == APPLICATION_XML || typ == TEXT_XML_UTF8 || typ == TEXT_XML
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodingType {
    Json,
    Xml,
}

pub enum DeserializableRejection {
    Json(JsonRejection),
    Xml(XmlRejection),
}

impl From<JsonRejection> for DeserializableRejection {
    fn from(v: JsonRejection) -> Self {
        Self::Json(v)
    }
}

impl From<XmlRejection> for DeserializableRejection {
    fn from(v: XmlRejection) -> Self {
        Self::Xml(v)
    }
}

impl IntoResponse for DeserializableRejection {
    fn into_response(self) -> Response {
        match self {
            DeserializableRejection::Json(rejection) => rejection.into_response(),
            DeserializableRejection::Xml(rejection) => rejection.into_response(),
        }
    }
}

async fn get_types(req: &mut Request) -> anyhow::Result<(EncodingType, MediaTypeBuf)> {
    let TypedHeader(content_type) = req.extract_parts::<TypedHeader<ContentType>>().await?;

    let encoding_type = match content_type.0 {
        typ if is_json(&typ) => EncodingType::Json,
        typ if is_xml(&typ) => EncodingType::Xml,
        _ => EncodingType::Json,
    };

    let TypedHeader(header) = req.extract_parts::<TypedHeader<Accept>>().await?;

    let media_type = header
        .negotiate(SUPPORTED_MEDIA_TYPES)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Failed to negotiate content type"))?;

    Ok((encoding_type, media_type.into()))
}

pub struct Deserializable<T>(pub EncodingType, pub T)
where
    T: serde::de::DeserializeOwned;

#[async_trait::async_trait]
impl<S, T> FromRequest<S> for Deserializable<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = DeserializableRejection;

    async fn from_request(mut req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (encoding_type, media_type) = match get_types(&mut req).await {
            Ok(pair) => pair,
            Err(_) => (EncodingType::Json, ContentType::json().0),
        };

        if is_xml(&media_type) {
            let Xml(t) = Xml::<T>::from_request(req, state).await?;

            return Ok(Self(encoding_type, t));
        }

        let Json(t) = Json::<T>::from_request(req, state).await?;

        Ok(Self(encoding_type, t))
    }
}

pub struct Serializable<T>(pub EncodingType, pub T)
where
    T: serde::Serialize;

impl<T> IntoResponse for Serializable<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let Self(encoding_type, t) = self;

        match encoding_type {
            EncodingType::Json => Json(t).into_response(),
            EncodingType::Xml => Xml(t).into_response(),
        }
    }
}
