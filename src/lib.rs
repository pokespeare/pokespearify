pub mod api_clients;
pub mod config;
pub mod routes;
pub mod telemetry;

use std::net::{SocketAddr, ToSocketAddrs};

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::api_clients::pokeapi::PokeApi;
use crate::api_clients::shakespeare::TranslationApi;
use crate::config::{PokeApiUrl, TranslateApiUrl};
use crate::routes::healthz;
use crate::routes::pokemon::pokemon;

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
    pub async fn new<A>(
        addr: A,
        poke_api_url: PokeApiUrl,
        translate_api_url: TranslateApiUrl,
    ) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let poke_api = web::Data::new(PokeApi::new(poke_api_url));
        let translate_api = web::Data::new(TranslationApi::new(translate_api_url));
        let srv = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger)
                .route("/healthz", web::get().to(healthz))
                .app_data(poke_api.clone())
                .app_data(translate_api.clone())
                .route("/pokemon/{pokemon_name}", web::get().to(pokemon))
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
