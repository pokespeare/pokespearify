pub mod api_clients;
pub mod config;
pub mod routes;
pub mod telemetry;

use std::net::{SocketAddr, ToSocketAddrs};

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use routes::healthz;

/// The Pokespeare Application.
///
/// This struct wraps an `actix_web::dev::Server` handling requests to the service.
pub struct Application {
    server: Server,
    addr: SocketAddr,
}

impl Application {
    /// Construct the Application and start the response handlers
    ///
    /// This method only constructs and starts the HTTP server, it then returns the Server handle.
    /// The `Application::run()` method can be used to await the server exit.
    pub async fn new<A>(addr: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let srv = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger)
                .route("/healthz", web::get().to(healthz))
        })
        .bind(addr)?;

        let addrs = srv.addrs();
        let addr = addrs[0];

        Ok(Application {
            server: srv.run(),
            addr,
        })
    }

    /// Run the Server until exit.
    pub async fn run(self) -> std::io::Result<()> {
        self.server.await
    }

    /// Get the `SocketAddr`s this server is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
