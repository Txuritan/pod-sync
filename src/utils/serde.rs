use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    Json, RequestExt,
};
use axum_extra::TypedHeader;
use headers_accept::Accept;
use mediatype::{
    names::{APPLICATION, CHARSET, JSON, TEXT, XML}, values::UTF_8, MediaType
};

use crate::utils::{content_type::ContentType, xml::Xml};

pub const APPLICATION_JSON: MediaType<'static> = MediaType::new(APPLICATION, JSON);
pub const APPLICATION_XML_UTF8: MediaType<'static> = MediaType::from_parts(APPLICATION, XML, None, &[(CHARSET, UTF_8)]);
pub const APPLICATION_XML: MediaType<'static> = MediaType::new(APPLICATION, XML); // TODO: restrict XML to UTF-8?
pub const TEXT_XML_UTF8: MediaType<'static> = MediaType::from_parts(TEXT, XML, None, &[(CHARSET, UTF_8)]);
pub const TEXT_XML: MediaType<'static> = MediaType::new(TEXT, XML); // TODO: restrict XML to UTF-8?

const SUPPORTED_MEDIA_TYPES: &[MediaType<'static>] = &[APPLICATION_JSON, APPLICATION_XML_UTF8, TEXT_XML_UTF8, APPLICATION_XML, TEXT_XML];

fn is_json(typ: &MediaType) -> bool {
    typ == &APPLICATION_JSON
}

fn is_xml(typ: &MediaType) -> bool {
    typ == &APPLICATION_XML_UTF8 || typ == &APPLICATION_XML || typ == &TEXT_XML_UTF8 || typ == &TEXT_XML
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodingType {
    Json,
    Xml,
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializableRejection {}

impl IntoResponse for DeserializableRejection {
    fn into_response(self) -> Response {
        todo!()
    }
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
        let Ok(TypedHeader(content_type)) = req.extract_parts::<TypedHeader<ContentType>>().await else {
            return todo!();
        };

        let encoding_type = match content_type.0 {
            typ if is_json(&typ.to_ref()) => EncodingType::Json,
            typ if is_xml(&typ.to_ref()) => EncodingType::Xml,
            _ => EncodingType::Json,
        };

        let Ok(TypedHeader(header)) = req.extract_parts::<TypedHeader<Accept>>().await else {
            return todo!();
        };

        let Some(media_type) = header.negotiate(SUPPORTED_MEDIA_TYPES).cloned() else {
            return todo!();
        };

        if is_json(&media_type) {
            let t = match Json::<T>::from_request(req, state).await {
                Ok(Json(t)) => t,
                Err(_) => todo!(),
            };

            return Ok(Self(encoding_type, t));
        }
        if is_xml(&media_type) {
            let t = match Xml::<T>::from_request(req, state).await {
                Ok(Xml(t)) => t,
                Err(_) => todo!(),
            };

            return Ok(Self(encoding_type, t));
        }

        todo!()
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
