use actix_web::web;

use crate::docker::handlers;

/// Configure all docker management routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Dashboard
        .service(handlers::docker_manager)
        
        // Container management
        .service(handlers::list_containers)
        .service(handlers::start_container)
        .service(handlers::stop_container)
        .service(handlers::restart_container)
        .service(handlers::remove_container)
        .service(handlers::get_container_logs)
        
        // Image management
        .service(handlers::list_images)
        .service(handlers::remove_image)
        
        // Volume management
        .service(handlers::list_volumes)
        .service(handlers::remove_volume)
        
        // Network management
        .service(handlers::list_networks)
        .service(handlers::remove_network);
}
