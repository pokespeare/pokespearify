use actix_web::web::{self, HttpResponse};
use rand::{thread_rng, Rng};
use serde::Serialize;

use crate::api_clients::pokeapi::PokeApi;
use crate::api_clients::shakespeare::TranslationApi;
use crate::api_clients::ApiError;

/// Handler for the Shakespeare meets Pokémon endpoint.
///
/// Given a Pokémon name in the path, it returns a shakespeare-ified description of the
/// Pokémon species.
///
/// # Implementation Detail
///
/// The translation is based on api.funtranslations.com which has strict rate-limitting.
/// We only get up to 5 requests per hour and 60 per day on the free tier. Upon reaching
/// the rate limit, we forward the 429 status to the caller.
#[tracing::instrument(
    name = "Return a shakespeared Pokémon description",
    skip(poke_api, translate_api),
    fields(
        poke_api_url = %poke_api.base_url(),
        translate_api_url = %translate_api.base_url(),
    )
)]
pub async fn pokemon(
    pokemon_name: web::Path<String>,
    poke_api: web::Data<PokeApi>,
    translate_api: web::Data<TranslationApi>,
) -> Result<HttpResponse, HttpResponse> {
    let pokemon_response = poke_api
        .get_pokemon_species_description(&*pokemon_name)
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    // always returning the same trivia is boring, mix it up a bit through randomization
    let mut rng = thread_rng();
    let english_flavor_texts = pokemon_response
        .english_flavor_text_entries()
        .collect::<Vec<_>>();
    let choice_idx = rng.gen_range(0..english_flavor_texts.len());
    let choice = english_flavor_texts[choice_idx];

    let translation = translate_api
        .translate(choice.flavor_text())
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            // funtranslations API has a strict RateLimit on the free tier with max 5/h
            // provide some
            if matches!(&e, &ApiError::RateLimit(_)) {
                return HttpResponse::TooManyRequests().json("Too many requests, try again later.");
            }
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(ShakespearedDescription::new(
        pokemon_name.into_inner(),
        translation,
    )))
}

#[derive(Serialize, Debug)]
pub struct ShakespearedDescription {
    name: String,
    description: String,
}

impl ShakespearedDescription {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}
