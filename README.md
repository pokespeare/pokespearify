# Shakespearified Pokémon Descriptions

This service fetches Pokémon Species descriptions from https://pokeapi.co and Shakespeare-ifies them via https://api.funtranslations.com/.
Since a given species can have lots of descriptions, a random one is chosen each time.

# Notes

## Caching

The Fun Translations API has a fairly strict rate limit with 5 calls per hour. This could be mitigated through some caching layer since
any matching texts are expected to receive the same translation. The implementation could use some (forward) proxy with a persistent
key-value store to keep the cached translations between instantiations. A less involved solution could ise an in-memory LRU, e.g. via
the `cached` crate.

I made the choice to randomly select the description text from the available texts, so I decided against a caching reverse proxy which
would remove this randomness.

Although the PokéApi doesn't appear to have strict rate-limitting, the returned data is pretty large at times. A cache could also yield
some improvements here.

Investing some more time, I'd explore caching solutions which the service itself could be agnostic to.

## Input Validation

It's possible to retrieve the raw CSV files `pokeapi.co` uses to populate their databases from GitHub. Using these resources, it's
possible to create lookups for known Pokémon at startup and fail requests for unknown Pokémon before hitting the external API. I decided
against this path since it'd increase the maintenance burden as there are no stability guarantees regarding the location of resources on
GitHub and statically copying the data once could lead to out-of-sync data when a new generation is released, a typo is fixed or anything
else would be changed.

# Setup

There are two ways to run this service, either containerized via docker or directly on the host.

## Host Setup

Running this service directly on the host requires Rust [(How to install)](https://rustup.rs).

With Rust installed, the service can be cloned and started by running the following commands:

~~~sh
git clone https://github.com/pokespeare/pokespearify
cd pokespearify
cargo run --release
~~~

By default, the service listens on Port 5000. This can either be modified through the `APP_PORT` environment variable or through the
`config.yml` file in the root directory of this repository.

## Docker Setup

Docker [(Get started)](https://www.docker.com/get-started) is the prerequisite for this setup. After installing, it should be possible
to run via:

~~~sh
git clone https://github.com/pokespeare/pokespearify
cd pokespearify
# build & tag
docker build --tag pokespearify --file Dockerfile .
# run the service with automated cleanup
CONTAINER_APP_PORT=5000
HOST_PORT=5000
docker run --rm -p $HOST_PORT:$CONTAINER_APP_PORT --env APP_PORT=$CONTAINER_APP_PORT pokespearify
~~~

## Usage

The service is then reachable through e.g.:

~~~sh
# note that the PokéApi currently expects all names in lower-case.
$ curl http://localhost:5000/pokemon/charizard
{"name":"charizard","description":"Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally."}
~~~

The above assumes `HOST_PORT=5000` in the containerized version.
