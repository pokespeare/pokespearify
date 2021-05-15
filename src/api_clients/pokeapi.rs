use reqwest::Url;
use serde::Deserialize;

use crate::api_clients::ApiError;
use crate::config::PokeApiUrl;

/// API Client for pokeapi.co
///
/// Currently only a subset of the data for the `pokemon-species` is supported, i.e. only
/// information relevant for english flavor texts is kept.
#[derive(Clone, Debug)]
pub struct PokeApi {
    client: reqwest::Client,
    base_url: Url,
}

impl PokeApi {
    const SPECIES: &'static str = "api/v2/pokemon-species/";
    /// Construct a new PokeAPI client.
    pub fn new(base_url: PokeApiUrl) -> Self {
        PokeApi {
            client: reqwest::Client::new(),
            base_url: base_url.0,
        }
    }

    /// Makes a call to the Pokemon Species Endpoint.
    ///
    /// The returned object only contains the fields relevant for the Shakespeareation service.
    #[tracing::instrument(name = "Get pokemon description", skip(self))]
    pub async fn get_pokemon_species_description(
        &self,
        pokemon: &str,
    ) -> Result<PokemonSpeciesResponse, ApiError> {
        let url = self
            .base_url
            .join(Self::SPECIES)
            .and_then(|url| url.join(pokemon))?;

        let resp = self.client.get(url).send().await?;
        resp.error_for_status_ref()?;
        Ok(resp.json().await?)
    }

    /// Get the base URL of the PokéAPI.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

/// Response model for the Pokemon Species endpoint.
#[derive(Deserialize, Debug, PartialEq)]
pub struct PokemonSpeciesResponse {
    // Most of the returned data is irrelevant for this service, thus it only contains the necessary bits.
    flavor_text_entries: Vec<FlavourTextEntry>,
}

impl PokemonSpeciesResponse {
    /// Convenience method for the Shakespeare translator to obtain an English flavor text.
    pub fn english_flavor_text_entries(&self) -> impl Iterator<Item = &FlavourTextEntry> {
        self.flavor_text_entries
            .iter()
            .filter(|e| matches!(&*e.language.name, "en"))
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct FlavourTextEntry {
    flavor_text: String,
    language: PokeApiLanguage,
}

impl FlavourTextEntry {
    /// Get a reference to the flavour text entry's flavor text.
    pub fn flavor_text(&self) -> &str {
        &self.flavor_text
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PokeApiLanguage {
    name: String,
}

#[cfg(test)]
mod test {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::config::PokeApiUrl;

    use super::{PokeApi, PokemonSpeciesResponse};

    static CHARIZARD_RESPONSE: &'static [u8] = include_bytes!("../../testdata/charizard.json");
    static PIKACHU_RESPONSE: &'static [u8] = include_bytes!("../../testdata/pikachu.json");

    #[tokio::test]
    async fn test_charizard() {
        let mock_server = MockServer::start().await;

        // set up our mock charizard
        let charizard_path = format!("/{}charizard/", PokeApi::SPECIES);
        Mock::given(method("GET"))
            .and(path(charizard_path))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(CHARIZARD_RESPONSE, "application/json"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        // can't leave out Pikachu
        let pikachu_path = format!("/{}pikachu/", PokeApi::SPECIES);
        Mock::given(method("GET"))
            .and(path(pikachu_path))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(PIKACHU_RESPONSE, "application/json"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let addr = mock_server.uri();
        let api = PokeApi::new(PokeApiUrl(addr.parse().unwrap()));

        let charizard_desc = api
            .get_pokemon_species_description("charizard/")
            .await
            .unwrap();
        let expected: PokemonSpeciesResponse = serde_json::from_slice(CHARIZARD_RESPONSE).unwrap();
        assert_eq!(charizard_desc, expected);

        let pikachu_desc = api
            .get_pokemon_species_description("pikachu/")
            .await
            .unwrap();
        let expected: PokemonSpeciesResponse = serde_json::from_slice(PIKACHU_RESPONSE).unwrap();
        assert_eq!(pikachu_desc, expected);
    }

    #[test]
    fn test_english_entries() {
        let resp: PokemonSpeciesResponse = serde_json::from_slice(CHARIZARD_RESPONSE).unwrap();
        assert!(resp
            .english_flavor_text_entries()
            .all(|e| matches!(&*e.language.name, "en")));
    }
}
