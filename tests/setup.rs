#![allow(dead_code)]
use lazy_static::lazy_static;
use pokespearify::telemetry::{get_subscriber, init_subscriber};

use pokespearify::Application;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

lazy_static! {
    static ref LOG: () = {
        let subscriber = get_subscriber("pokespeare-test-subscriber".into(), "error".into());
        init_subscriber(subscriber);
    };
}

static SINGLE_CHARIZARD_RESPONSE: &'static [u8] =
    include_bytes!("../testdata/charizard_single_text.json");

pub struct TestApp {
    inner: Application,
    mock_poke_api: MockServer,
}

impl TestApp {
    /// Spawn the TestApp.
    ///
    /// The TestApp does not initialize any Mock Servers.
    pub async fn spawn() -> TestApp {
        let _log = *LOG;

        let mock_poke_api = MockServer::start().await;
        let addr = ("127.0.0.1", 0);
        TestApp {
            inner: Application::new(addr, mock_poke_api.uri().parse().unwrap())
                .await
                .expect("Failed to start test Application"),
            mock_poke_api,
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
}
