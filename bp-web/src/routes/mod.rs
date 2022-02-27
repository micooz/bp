use crate::{
    controllers::{
        AssetsController, ConfigurationController, SecurityController, ServiceController, SystemInfoController,
    },
    state::State,
};

pub fn register(app: &mut tide::Server<State>) {
    // configuration
    app.at("/api/configuration/query").get(ConfigurationController::query);
    app.at("/api/configuration/create")
        .post(ConfigurationController::create);
    app.at("/api/configuration/modify")
        .post(ConfigurationController::modify);

    // security
    app.at("/api/security/query").get(SecurityController::query);
    app.at("/api/security/create").post(SecurityController::create);

    // system
    app.at("/api/system/info").get(SystemInfoController::info);

    // service
    app.at("/api/service/query").get(ServiceController::query);
    app.at("/api/service/start").post(ServiceController::start);
    app.at("/api/service/stop").post(ServiceController::stop);

    // static files
    app.at("/static/*").get(AssetsController::statics);
    app.at("/index.html").get(AssetsController::index);
    app.at("/").get(AssetsController::index);
}
