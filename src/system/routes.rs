use actix_web::web;
use super::handlers;

/// Configure all system monitoring routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Root and dashboard
        .service(handlers::index)
        .service(handlers::dashboard)
        
        // System information endpoints
        .service(handlers::get_system_info)
        .service(handlers::get_cpu_info)
        .service(handlers::get_cpu_usage)
        .service(handlers::get_memory_info)
        .service(handlers::get_disks_info)
        .service(handlers::get_network_info)
        .service(handlers::get_processes)
        .service(handlers::get_load_average)
        
        // Health and process management
        .service(handlers::health_check)
        .service(handlers::kill_process);
}
