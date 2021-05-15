#![allow(dead_code)]
use lazy_static::lazy_static;
use pokespearify::api_clients::shakespeare::{TranslationRequest, TranslationResponse};
use pokespearify::config::{PokeApiUrl, TranslateApiUrl};
use pokespearify::telemetry::{get_subscriber, init_subscriber};

use pokespearify::Application;
use wiremock::matchers::{any, body_json, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

lazy_static! {
    static ref LOG: () = {
        let subscriber = get_subscriber("pokespeare-test-subscriber".into(), "error".into());
        init_subscriber(subscriber);
    };
}

static SINGLE_CHARIZARD_RESPONSE: &'static [u8] =
    include_bytes!("../testdata/charizard_single_text.json");
static CHARIZARD_TRANSLATED_RESPONSE: &'static [u8] =
    include_bytes!("../testdata/charizard_single_translation.json");

pub struct TestApp {
    inner: Application,
    mock_poke_api: MockServer,
    mock_translate_api: MockServer,
}

impl TestApp {
    /// Spawn the TestApp.
    ///
    /// The TestApp does not initialize any Mock Servers.
    pub async fn spawn() -> TestApp {
        let _log = *LOG;

        let mock_poke_api = MockServer::start().await;
        let mock_translate_api = MockServer::start().await;
        let poke_api_url = PokeApiUrl(mock_poke_api.uri().parse().unwrap());
        let translate_api_url = TranslateApiUrl(mock_translate_api.uri().parse().unwrap());
        let addr = ("127.0.0.1", 0);
        TestApp {
            inner: Application::new(addr, poke_api_url, translate_api_url)
                .await
                .expect("Failed to start test Application"),
            mock_poke_api,
            mock_translate_api,
        }
    }

    /// Get a reference to the underlying `Application`.
    pub fn inner(&self) -> &Application {
        &self.inner
    }

    /// Mock the PokéApi by returning the same charizard description for all Pokémon queries.
    pub async fn with_poke_api(&self, expect: u64) -> &Self {
        Mock::given(method("GET"))
            .and(path_regex(r"api/v2/pokemon-species/([a-zA-Z]|-)+")) // it's all Charizard for testing purposes
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(SINGLE_CHARIZARD_RESPONSE, "application/json"),
            )
            .expect(expect)
            .mount(&self.mock_poke_api)
            .await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .mount(&self.mock_poke_api)
            .await;
        self
    }

    /// Mock the PokéApi by returning the same charizard description for all Pokémon queries.
    pub async fn with_translate_api(&self, expect: u64) -> &Self {
        Mock::given(method("POST"))
            .and(path("translate/shakespeare.json")) // it's all Charizard for testing purposes
            .and(body_json(TranslationRequest::from("Spits fire that\nis hot enough to\nmelt boulders.\u{0C}Known to cause\nforest fires\nunintentionally.")
            ))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(CHARIZARD_TRANSLATED_RESPONSE, "application/json"),
            ).expect(expect)
            .mount(&self.mock_translate_api).await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .mount(&self.mock_translate_api)
            .await;
        self
    }

    /// Mock the PokéApi and return TOO_MANY_REQUESTS 429 for everything.
    pub async fn with_translate_rate_limit(&self) -> &Self {
        Mock::given(method("POST"))
            .and(path("translate/shakespeare.json"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&self.mock_translate_api)
            .await;
        self
    }

    /// Convenience method to get the translated charizard description from the testdata.
    pub fn charizard_translation_response() -> TranslationResponse {
        serde_json::from_slice(CHARIZARD_TRANSLATED_RESPONSE)
            .expect("Failed to decode translated charizard description")
    }
}
