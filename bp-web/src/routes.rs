use crate::controllers::{AssetsController, ConfigurationController, DebugController};

pub fn register(app: &mut tide::Server<()>) {
    app.at("/api/configuration/query").get(ConfigurationController::query);
    app.at("/api/configuration/create")
        .post(ConfigurationController::create);
    app.at("/api/configuration/modify")
        .post(ConfigurationController::modify);

    app.at("/static/*").get(AssetsController::statics);
    app.at("/index.html").get(AssetsController::index);
    app.at("/").get(AssetsController::index);

    app.at("/").all(DebugController::index);
}
