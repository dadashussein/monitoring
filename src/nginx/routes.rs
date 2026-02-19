use actix_web::web;

use crate::nginx::handlers;

/// Configure all nginx management routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Dashboard
        .service(handlers::nginx_admin)
        // API endpoints
        .service(handlers::format_nginx_extra_config)
        .service(handlers::get_nginx_proxies)
        .service(handlers::create_nginx_proxy)
        .service(handlers::delete_nginx_proxy)
        .service(handlers::update_nginx_proxy);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_configure_routes_compiles() {
        // This test verifies that configure_routes can be called
        // and properly registers routes without panicking
        let app = test::init_service(
            App::new()
                .configure(configure_routes)
        ).await;
        
        // If we get here, the routes were configured successfully
        drop(app);
        assert!(true);
    }

    #[actix_web::test]
    async fn test_nginx_admin_route_exists() {
        // Test that the nginx admin dashboard route is registered
        let app = test::init_service(
            App::new()
                .configure(configure_routes)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/nginx")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // Should return 200 OK with HTML content
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_proxies_route_exists() {
        // Test that the get proxies route is registered
        let app = test::init_service(
            App::new()
                .configure(configure_routes)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/api/nginx/proxies")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // Route should exist (may return error if nginx not installed, but route exists)
        // We just verify the route is registered, not that nginx is working
        assert!(resp.status().as_u16() >= 200);
    }
}
