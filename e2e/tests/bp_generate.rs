use bp_cli::{
    commands::generate,
    options::generate::{ConfigType, GenerateOptions},
};
use e2e::fs;

#[tokio::test(flavor = "multi_thread")]
async fn test_generate_configs() {
    let config_path_client = "tests/tmp/client.config.json";
    let config_path_server = "tests/tmp/server.config.json";

    generate::run(GenerateOptions {
        config: Some(config_path_server.to_string()),
        config_type: ConfigType::Server,
        ..Default::default()
    })
    .await;

    generate::run(GenerateOptions {
        config: Some(config_path_client.to_string()),
        config_type: ConfigType::Client,
        ..Default::default()
    })
    .await;

    insta::assert_snapshot!("client", fs::read_file(config_path_client).await);
    insta::assert_snapshot!("server", fs::read_file(config_path_server).await);
}

// #[tokio::test(flavor = "multi_thread")]
// async fn test_generate_certificate() {

// }
