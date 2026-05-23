use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl<T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl<T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl<T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl<T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                }
                serde_json::Value::String(s) => {
                    params.push((format!("{}[{}]", prefix, key), s.clone()))
                }
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    vec![]
}

/// Internal use only
/// A content type supported by this client.
#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String),
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            Self::Json
        } else if content_type.starts_with("text/plain") {
            Self::Text
        } else {
            Self::Unsupported(content_type.to_string())
        }
    }
}

pub fn set_user_agent(
    req_builder: reqwest::RequestBuilder,
    user_agent: &Option<String>,
) -> reqwest::RequestBuilder {
    if let Some(ua) = user_agent {
        req_builder.header(reqwest::header::USER_AGENT, ua.clone())
    } else {
        req_builder
    }
}

pub fn apply_auth(
    req_builder: reqwest::RequestBuilder,
    config: &configuration::Configuration,
) -> reqwest::RequestBuilder {
    if let Some(ref token) = config.bearer_access_token {
        req_builder.bearer_auth(token)
    } else if let Some(ref token) = config.oauth_access_token {
        req_builder.bearer_auth(token)
    } else if let Some((ref user, ref pass)) = config.basic_auth {
        req_builder.basic_auth(user, pass.as_deref())
    } else if let Some(ref api_key) = config.api_key {
        let key = match &api_key.prefix {
            Some(prefix) => format!("{} {}", prefix, api_key.key),
            None => api_key.key.clone(),
        };
        req_builder.header(reqwest::header::AUTHORIZATION, key)
    } else {
        req_builder
    }
}

pub async fn parse_response<T, E>(resp: reqwest::Response) -> Result<T, Error<E>>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
{
    let status = resp.status();
    let content = resp.text().await?;
    if !status.is_client_error() && !status.is_server_error() {
        serde_json::from_str(&content).map_err(Error::from)
    } else {
        let entity = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

pub async fn parse_empty_response<E>(resp: reqwest::Response) -> Result<(), Error<E>>
where
    E: serde::de::DeserializeOwned,
{
    let status = resp.status();
    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

pub mod assertions_api;
pub mod auth_zen_service_api;
pub mod authorization_models_api;
pub mod relationship_queries_api;
pub mod relationship_tuples_api;
pub mod stores_api;

pub mod configuration;
