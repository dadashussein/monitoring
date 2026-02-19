use actix_web::{web, App, HttpServer, middleware};
use log::info;
use sysinfo::System;

// Import modules from lib.rs
use ubuntu_resource_api::config::AppConfig;
use ubuntu_resource_api::system::models::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Load configuration
    let config = AppConfig::from_env().unwrap_or_else(|_| AppConfig::with_defaults());
    let bind_addr = config.server.bind_address.clone();

    // Create shared application state
    let app_state = web::Data::new(AppState {
        system: std::sync::Mutex::new(System::new_all()),
        nginx_config: config.nginx.clone(),
        docker_config: config.docker.clone(),
    });

    info!("🚀 Ubuntu Resource API starting on http://{}", bind_addr);
    info!("📊 Dashboard: http://{}/dashboard", bind_addr);
    info!("🔄 Nginx Manager: http://{}/nginx", bind_addr);
    info!("🐳 Docker Manager: http://{}/docker", bind_addr);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .configure(ubuntu_resource_api::system::routes::configure_routes)
            .configure(ubuntu_resource_api::nginx::routes::configure_routes)
            .configure(ubuntu_resource_api::docker::routes::configure_routes)
    })
    .bind(bind_addr)?
    .run()
    .await
}
