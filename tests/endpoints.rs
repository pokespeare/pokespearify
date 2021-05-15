mod setup;

use reqwest::StatusCode;

use setup::TestApp;

#[actix_rt::test]
async fn test_health() {
    let app = TestApp::spawn().await;
    let resp = reqwest::get(format!("http://{}/healthz", app.inner().addr()))
        .await
        .expect("The healthcheck endpoint is not working");
    assert_eq!(resp.status(), StatusCode::OK);
}
