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
    app.with_poke_api(2).await;

    let resp = reqwest::get(format!("http://{}/pokemon/charizard", app.inner().addr()))
        .await
        .expect("The pokemon endpoint is not working");

    assert_eq!(resp.status(), StatusCode::OK);

    let resp = resp
        .json::<ShakespearedDescriptionResponse>()
        .await
        .expect("Got an invalid response");
    assert_eq!(resp.name, "charizard");

    // hard code the response to the original description until the translation is implemented
    let desc = "Spits fire that\nis hot enough to\nmelt boulders.\u{0C}Known to cause\nforest fires\nunintentionally.";
    assert_eq!(resp.description, desc);

    let resp = reqwest::get(format!("http://{}/pokemon/pikachu", app.inner().addr()))
        .await
        .expect("The pokemon endpoint is not working");
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = resp
        .json::<ShakespearedDescriptionResponse>()
        .await
        .expect("Got an invalid response");
    assert_eq!(resp.name, "pikachu");

    assert_eq!(resp.description, desc);
}
