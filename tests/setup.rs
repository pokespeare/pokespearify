#![allow(dead_code)]
use lazy_static::lazy_static;
use pokespearify::telemetry::{get_subscriber, init_subscriber};

use pokespearify::Application;

lazy_static! {
    static ref LOG: () = {
        let subscriber = get_subscriber("pokespeare-test-subscriber".into(), "error".into());
        init_subscriber(subscriber);
    };
}

pub struct TestApp {
    inner: Application,
}

impl TestApp {
    /// Spawn the TestApp.
    ///
    /// The TestApp does not initialize any Mock Servers.
    pub async fn spawn() -> TestApp {
        let _log = *LOG;

        let addr = ("127.0.0.1", 0);
        TestApp {
            inner: Application::new(addr)
                .await
                .expect("Failed to start test Application"),
        }
    }

    /// Get a reference to the underlying `Application`.
    pub fn inner(&self) -> &Application {
        &self.inner
    }
}
