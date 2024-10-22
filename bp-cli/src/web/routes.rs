use super::{
    common::state::State,
    controllers::{AssetsController, ConfigController, LoggingController, ServiceController, SystemInfoController},
    middlewares::crypto::CryptoMiddleware,
};
use crate::options::web::CryptoMethod;

pub fn register(app: &mut tide::Server<State>) {
    let state = app.state();

    // apply middleware
    if !matches!(state.opts.crypto, CryptoMethod::None) {
        app.with(CryptoMiddleware::default());
    }

    // configuration
    app.at("/api/config/query").get(ConfigController::query);
    app.at("/api/config/query_acl").get(ConfigController::query_acl);
    app.at("/api/config/create").post(ConfigController::create);
    app.at("/api/config/create_tls_config")
        .post(ConfigController::create_tls_config);
    app.at("/api/config/modify").post(ConfigController::modify);

    // logging
    app.at("/api/logging/tail").get(LoggingController::tail);

    // monitor
    app.at("/api/monitor/system/info").get(SystemInfoController::info);

    // service
    app.at("/api/service/query").get(ServiceController::query);
    app.at("/api/service/start").post(ServiceController::start);
    app.at("/api/service/stop").post(ServiceController::stop);

    // static files
    app.at("/static/*").get(AssetsController::statics);
    app.at("/index.html").get(AssetsController::index);
    app.at("/").get(AssetsController::index);
}
