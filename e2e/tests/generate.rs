use bp_cli::{
    commands::generate,
    options::generate::{ConfigType, GenerateOptions},
};
use e2e::fs;

#[tokio::test(flavor = "multi_thread")]
async fn test_generate_config() {
    // client config.json

    let config_path = "tests/tmp/client.config.json";

    generate::run(GenerateOptions {
        config: Some(config_path.to_string()),
        ..Default::default()
    })
    .await;

    insta::assert_debug_snapshot!(fs::read_file(config_path).await);

    // server config.yaml

    let config_path = "tests/tmp/server.config.yaml";

    generate::run(GenerateOptions {
        config: Some(config_path.to_string()),
        config_type: ConfigType::Server,
        ..Default::default()
    })
    .await;

    insta::assert_debug_snapshot!(fs::read_file(config_path).await);
}
