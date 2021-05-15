use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::api_clients::ApiError;
use crate::config::TranslateApiUrl;

/// API Client for Fun Translations' Shakespear translator.
#[derive(Clone, Debug)]
pub struct TranslationApi {
    client: reqwest::Client,
    base_url: Url,
}

impl TranslationApi {
    const SHAKESPEARE_TRANSLATOR: &'static str = "translate/shakespeare.json";

    /// Construct a new API client sending requests with the given base URL.
    pub fn new(base_url: TranslateApiUrl) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.0,
        }
    }

    /// Translate the input string to Shakespearean English.
    #[tracing::instrument(name = "Get translation description", skip(self))]
    pub async fn translate(&self, text: &str) -> Result<String, ApiError> {
        let url = self.base_url.join(Self::SHAKESPEARE_TRANSLATOR)?;
        let resp = self
            .client
            .post(url)
            .json(&TranslationRequest::from(text))
            .send()
            .await?;
        let status = resp.status();
        resp.error_for_status_ref().map_err(|e| {
            if status == StatusCode::TOO_MANY_REQUESTS {
                ApiError::RateLimit(e)
            } else {
                ApiError::Reqwest(e)
            }
        })?;

        let parsed_resp: TranslationResponse = resp.json().await?;
        Ok(parsed_resp.contents.translated)
    }

    /// Get the base URL of the API.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct TranslationRequest<'a> {
    text: &'a str,
}

impl<'a> From<&'a str> for TranslationRequest<'a> {
    fn from(text: &'a str) -> Self {
        TranslationRequest { text }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TranslationResponse {
    contents: TranslationContents,
}

impl TranslationResponse {
    pub fn translation(&self) -> &str {
        &*self.contents.translated
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TranslationContents {
    translated: String,
}

#[cfg(test)]
mod test {
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::api_clients::ApiError;
    use crate::config::TranslateApiUrl;

    use super::{TranslationApi, TranslationRequest};

    static TRANSLATED_RESPONSE: &'static [u8] = include_bytes!("../../testdata/shakespeare.json");

    #[tokio::test]
    async fn test_shakespeare_api() {
        let mock_server = MockServer::start().await;
        let mock_path = format!("/{}", TranslationApi::SHAKESPEARE_TRANSLATOR);
        Mock::given(method("POST"))
            .and(path(mock_path))
            .and(body_json(&TranslationRequest {
                text: "You gave Mr. Tim a hearty meal, but unfortunately what he ate made him die.",
            }))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(TRANSLATED_RESPONSE, "application/json"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let addr = mock_server.uri();
        let api = TranslationApi::new(TranslateApiUrl(addr.parse().unwrap()));

        let resp = api
            .translate(
                "You gave Mr. Tim a hearty meal, but unfortunately what he ate made him die.",
            )
            .await
            .unwrap();
        assert_eq!(resp, "Thee did giveth mr. Tim a hearty meal,  but unfortunately what he did doth englut did maketh him kicketh the bucket.");
    }

    #[tokio::test]
    async fn test_shakespeare_api_ratelimit() {
        let mock_server = MockServer::start().await;
        let mock_path = format!("/{}", TranslationApi::SHAKESPEARE_TRANSLATOR);
        Mock::given(method("POST"))
            .and(path(mock_path))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&mock_server)
            .await;

        let addr = mock_server.uri();
        let api = TranslationApi::new(TranslateApiUrl(addr.parse().unwrap()));

        let resp = api
            .translate(
                "You gave Mr. Tim a hearty meal, but unfortunately what he ate made him die.",
            )
            .await
            .expect_err("The API call should have returned an error");
        assert!(matches!(resp, ApiError::RateLimit(_)));
    }

    #[tokio::test]
    async fn test_shakespeare_api_internal_error() {
        let mock_server = MockServer::start().await;
        let mock_path = format!("/{}", TranslationApi::SHAKESPEARE_TRANSLATOR);
        Mock::given(method("POST"))
            .and(path(mock_path))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let addr = mock_server.uri();
        let api = TranslationApi::new(TranslateApiUrl(addr.parse().unwrap()));

        let resp = api
            .translate(
                "You gave Mr. Tim a hearty meal, but unfortunately what he ate made him die.",
            )
            .await
            .expect_err("The API call should have returned an error");
        assert!(matches!(&resp, &ApiError::Reqwest(_)));
    }
}
