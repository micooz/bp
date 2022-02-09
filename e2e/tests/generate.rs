use bp_cli::{
    commands::generate,
    options::generate::{ConfigType, GenerateOptions},
};
use e2e::fs;

#[tokio::test(flavor = "multi_thread")]
async fn test_generate_client_config() {
    let config_path = "tests/tmp/client.config.json";

    generate::run(GenerateOptions {
        config: Some(config_path.to_string()),
        ..Default::default()
    })
    .await;

    insta::assert_json_snapshot!(fs::read_file(config_path).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_generate_server_config() {
    let config_path = "tests/tmp/server.config.yaml";

    generate::run(GenerateOptions {
        config: Some(config_path.to_string()),
        config_type: ConfigType::Server,
        ..Default::default()
    })
    .await;

    insta::assert_yaml_snapshot!(fs::read_file(config_path).await);
}
