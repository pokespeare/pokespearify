mod setup;

use reqwest::StatusCode;
use serde::Deserialize;

use setup::TestApp;

#[actix_rt::test]
async fn test_health() {
    let app = TestApp::spawn().await;
    let resp = reqwest::get(format!("http://{}/healthz", app.inner().addr()))
        .await
        .expect("The healthcheck endpoint is not working");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_pokemon() {
    #[derive(Deserialize, Debug)]
    struct ShakespearedDescriptionResponse {
        name: String,
        description: String,
    }

    let app = TestApp::spawn().await;
    app.with_poke_api(2).await.with_translate_api(2).await;

    let resp = reqwest::get(format!("http://{}/pokemon/charizard", app.inner().addr()))
        .await
        .expect("The pokemon endpoint is not working");

    assert_eq!(resp.status(), StatusCode::OK);

    let resp = resp
        .json::<ShakespearedDescriptionResponse>()
        .await
        .expect("Got an invalid response");
    assert_eq!(resp.name, "charizard");

    let desc = TestApp::charizard_translation_response();
    assert_eq!(resp.description, desc.translation());

    let resp = reqwest::get(format!("http://{}/pokemon/pikachu", app.inner().addr()))
        .await
        .expect("The pokemon endpoint is not working");
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = resp
        .json::<ShakespearedDescriptionResponse>()
        .await
        .expect("Got an invalid response");
    assert_eq!(resp.name, "pikachu");

    let desc = TestApp::charizard_translation_response();
    assert_eq!(resp.description, desc.translation());
}

#[actix_rt::test]
async fn test_rate_limit() {
    let app = TestApp::spawn().await;
    app.with_poke_api(1).await.with_translate_rate_limit().await;

    let resp = reqwest::get(format!("http://{}/pokemon/charizard", app.inner().addr()))
        .await
        .expect("The pokemon endpoint is not working");
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    let resp = resp
        .json::<String>()
        .await
        .expect("Got an invalid response");
    assert_eq!(resp, "Too many requests, try again later.");
}

#[derive(Deserialize, Debug)]
pub struct ShakespearedDescriptionResponse {
    name: String,
    description: String,
}
