use actix_web::{web, HttpResponse, Responder};
use log::{info, warn, error};
use std::fs;
use std::path::Path;

use crate::nginx::models::{NginxProxy, NginxResponse, FormatRequest, FormatResponse};
use crate::nginx::config::{validate_nginx_extra_config, generate_nginx_config};
use crate::system::models::AppState;

const NGINX_ADMIN_HTML: &str = include_str!("../templates/nginx_admin.html");

/// Serve the nginx admin HTML dashboard
#[actix_web::get("/nginx")]
pub async fn nginx_admin() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(NGINX_ADMIN_HTML)
}

/// Format and validate nginx extra configuration
#[actix_web::post("/api/nginx/format")]
pub async fn format_nginx_extra_config(req: web::Json<FormatRequest>) -> impl Responder {
    info!("POST /api/nginx/format - Formatting nginx config");
    
    match validate_nginx_extra_config(&req.config) {
        Ok(formatted) => {
            HttpResponse::Ok().json(FormatResponse {
                success: true,
                formatted: Some(formatted),
                error: None,
            })
        },
        Err(e) => {
            HttpResponse::Ok().json(FormatResponse {
                success: false,
                formatted: None,
                error: Some(e),
            })
        }
    }
}

/// List all nginx proxy configurations
#[actix_web::get("/api/nginx/proxies")]
pub async fn get_nginx_proxies(data: web::Data<AppState>) -> impl Responder {
    info!("GET /api/nginx/proxies - Listing nginx configurations");
    
    let nginx_sites_available = &data.nginx_config.sites_available_path;
    let mut proxies = Vec::new();
    
    // Check if directory exists
    if !Path::new(nginx_sites_available).exists() {
        warn!("Nginx sites-available directory not found: {}", nginx_sites_available);
        return HttpResponse::Ok().json(serde_json::json!({
            "proxies": proxies,
            "warning": format!("Nginx konfiqurasiya qovluğu tapılmadı: {}. Nginx quraşdırılıb?", nginx_sites_available)
        }));
    }
    
    match fs::read_dir(nginx_sites_available) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(name) = entry.file_name().to_str() {
                            // Skip default and example configs
                            if name == "default" || name.contains("example") {
                                continue;
                            }
                            
                            let domain = content.lines()
                                .find(|l| l.trim().starts_with("server_name"))
                                .and_then(|l| l.split_whitespace().nth(1))
                                .unwrap_or("unknown")
                                .trim_end_matches(';')
                                .to_string();
                            
                            let backend = content.lines()
                                .find(|l| l.trim().starts_with("proxy_pass"))
                                .and_then(|l| l.split_whitespace().nth(1))
                                .unwrap_or("unknown")
                                .trim_end_matches(';')
                                .to_string();
                            
                            let ssl = content.contains("listen 443 ssl");
                            
                            info!("Found proxy config: {} -> {} ({})", name, domain, backend);
                            
                            proxies.push(NginxProxy {
                                name: name.to_string(),
                                domain,
                                backend,
                                ssl,
                                extra_config: None,
                            });
                        }
                    } else {
                        warn!("Could not read file: {:?}", path);
                    }
                }
            }
            info!("Found {} proxy configurations", proxies.len());
        },
        Err(e) => {
            error!("Failed to read nginx directory: {}", e);
            return HttpResponse::Ok().json(serde_json::json!({
                "proxies": proxies,
                "error": format!("Qovluq oxuna bilmədi: {}. İcazə problemi ola bilər.", e)
            }));
        }
    }
    
    HttpResponse::Ok().json(proxies)
}

/// Create a new nginx proxy configuration
#[actix_web::post("/api/nginx/proxies")]
pub async fn create_nginx_proxy(data: web::Data<AppState>, proxy: web::Json<NginxProxy>) -> impl Responder {
    let proxy_domain = proxy.domain.clone();
    info!("POST /api/nginx/proxies - Creating proxy: {} -> {}", proxy_domain, proxy.backend);
    
    let nginx_sites_available = &data.nginx_config.sites_available_path;
    let nginx_sites_enabled = &data.nginx_config.sites_enabled_path;
    
    // Validate and format extra_config if provided
    let validated_proxy = if let Some(extra) = &proxy.extra_config {
        if !extra.trim().is_empty() {
            match validate_nginx_extra_config(extra) {
                Ok(formatted) => {
                    info!("Extra config validated and formatted");
                    NginxProxy {
                        name: proxy.name.clone(),
                        domain: proxy.domain.clone(),
                        backend: proxy.backend.clone(),
                        ssl: proxy.ssl,
                        extra_config: Some(formatted),
                    }
                },
                Err(e) => {
                    error!("Extra config validation failed: {}", e);
                    return HttpResponse::BadRequest().json(NginxResponse {
                        success: false,
                        message: format!("❌ Əlavə konfiqurasiya xətası: {}", e),
                    });
                }
            }
        } else {
            proxy.into_inner()
        }
    } else {
        proxy.into_inner()
    };
    
    // Check if nginx directories exist
    if !Path::new(nginx_sites_available).exists() {
        error!("Nginx sites-available directory not found");
        return HttpResponse::InternalServerError().json(NginxResponse {
            success: false,
            message: format!("Nginx qovluğu tapılmadı: {}. Nginx quraşdırılıb?", nginx_sites_available),
        });
    }
    
    let config_path = format!("{}/{}", nginx_sites_available, validated_proxy.name);
    let enabled_path = format!("{}/{}", nginx_sites_enabled, validated_proxy.name);
    let backup_path = format!("{}/{}.backup", nginx_sites_available, validated_proxy.name);
    
    info!("Config path: {}", config_path);
    info!("Enabled path: {}", enabled_path);
    
    // Backup existing config if it exists
    let had_backup = if Path::new(&config_path).exists() {
        info!("Backing up existing config");
        match fs::copy(&config_path, &backup_path) {
            Ok(_) => {
                info!("Backup created successfully");
                true
            },
            Err(e) => {
                error!("Failed to create backup: {}", e);
                return HttpResponse::InternalServerError().json(NginxResponse {
                    success: false,
                    message: format!("Mövcud konfiqurasiya yedəklənə bilmədi: {}. Dəyişiklik təhlükəlidir.", e),
                });
            }
        }
    } else {
        false
    };
    
    // Generate nginx config
    let config_content = generate_nginx_config(&validated_proxy);
    
    // Write config file
    match fs::write(&config_path, &config_content) {
        Ok(_) => {
            info!("Config file written successfully");
            
            // Create symlink to enable
            if Path::new(&enabled_path).exists() {
                info!("Removing existing symlink");
                let _ = fs::remove_file(&enabled_path);
            }
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                if let Err(e) = symlink(&config_path, &enabled_path) {
                    error!("Failed to create symlink: {}", e);
                    // Restore backup if we had one
                    if had_backup {
                        let _ = fs::copy(&backup_path, &config_path);
                        let _ = fs::remove_file(&backup_path);
                    }
                    return HttpResponse::InternalServerError().json(NginxResponse {
                        success: false,
                        message: format!("Symlink yaradıla bilmədi: {}. Root icazəsi lazımdır.", e),
                    });
                }
                info!("Symlink created successfully");
            }
            
            // Test nginx config
            info!("Testing nginx configuration...");
            let output = std::process::Command::new("nginx")
                .args(&["-t"])
                .output();
            
            match output {
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    info!("Nginx test output: {}", stderr);
                    
                    if result.status.success() {
                        // Test passed - remove backup and reload nginx
                        if had_backup {
                            let _ = fs::remove_file(&backup_path);
                            info!("Backup removed after successful test");
                        }
                        
                        info!("Reloading nginx...");
                        let reload = std::process::Command::new("systemctl")
                            .args(&["reload", "nginx"])
                            .output();
                        
                        match reload {
                            Ok(reload_result) => {
                                if reload_result.status.success() {
                                    info!("Nginx reloaded successfully");
                                    HttpResponse::Ok().json(NginxResponse {
                                        success: true,
                                        message: format!("✅ {} proxy konfiqurasiyası yaradıldı və aktiv edildi", validated_proxy.domain),
                                    })
                                } else {
                                    let reload_err = String::from_utf8_lossy(&reload_result.stderr);
                                    error!("Nginx reload failed: {}", reload_err);
                                    HttpResponse::InternalServerError().json(NginxResponse {
                                        success: false,
                                        message: format!("Nginx reload edilə bilmədi: {}. Systemctl icazəsi lazımdır.", reload_err),
                                    })
                                }
                            },
                            Err(e) => {
                                error!("Failed to execute systemctl: {}", e);
                                HttpResponse::InternalServerError().json(NginxResponse {
                                    success: false,
                                    message: format!("Systemctl çalışdırıla bilmədi: {}. Root icazəsi lazımdır.", e),
                                })
                            }
                        }
                    } else {
                        // Test failed - restore backup if we had one
                        error!("Nginx config test failed, rolling back");
                        
                        if had_backup {
                            info!("Restoring backup config");
                            match fs::copy(&backup_path, &config_path) {
                                Ok(_) => {
                                    let _ = fs::remove_file(&backup_path);
                                    info!("Backup restored successfully");
                                    HttpResponse::BadRequest().json(NginxResponse {
                                        success: false,
                                        message: format!("❌ Nginx konfiqurasiya xətası: {}\n\n✅ Əvvəlki konfiqurasiya bərpa edildi.", stderr),
                                    })
                                },
                                Err(e) => {
                                    error!("Failed to restore backup: {}", e);
                                    HttpResponse::InternalServerError().json(NginxResponse {
                                        success: false,
                                        message: format!("❌ Nginx xətası: {}\n❌ Yedək bərpa edilə bilmədi: {}", stderr, e),
                                    })
                                }
                            }
                        } else {
                            // No backup - just remove the bad config
                            let _ = fs::remove_file(&config_path);
                            let _ = fs::remove_file(&enabled_path);
                            HttpResponse::BadRequest().json(NginxResponse {
                                success: false,
                                message: format!("❌ Nginx konfiqurasiya xətası: {}\n\nYanlış konfiqurasiya silindi.", stderr),
                            })
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to test nginx config: {}", e);
                    // Restore backup if we had one
                    if had_backup {
                        let _ = fs::copy(&backup_path, &config_path);
                        let _ = fs::remove_file(&backup_path);
                    }
                    HttpResponse::InternalServerError().json(NginxResponse {
                        success: false,
                        message: format!("Nginx test edilə bilmədi: {}. Nginx quraşdırılıb?", e),
                    })
                }
            }
        },
        Err(e) => {
            error!("Failed to write config file: {}", e);
            // Restore backup if we had one
            if had_backup {
                let _ = fs::copy(&backup_path, &config_path);
                let _ = fs::remove_file(&backup_path);
            }
            HttpResponse::InternalServerError().json(NginxResponse {
                success: false,
                message: format!("Fayl yazıla bilmədi: {}. Root icazəsi lazımdır. Container root olaraq çalışmalıdır.", e),
            })
        },
    }
}

/// Delete an nginx proxy configuration
#[actix_web::delete("/api/nginx/proxies/{name}")]
pub async fn delete_nginx_proxy(data: web::Data<AppState>, name: web::Path<String>) -> impl Responder {
    info!("DELETE /api/nginx/proxies/{} - Deleting proxy", name);
    
    let nginx_sites_available = &data.nginx_config.sites_available_path;
    let nginx_sites_enabled = &data.nginx_config.sites_enabled_path;
    
    let config_path = format!("{}/{}", nginx_sites_available, name.as_str());
    let enabled_path = format!("{}/{}", nginx_sites_enabled, name.as_str());
    
    // Remove symlink
    if Path::new(&enabled_path).exists() {
        if let Err(e) = fs::remove_file(&enabled_path) {
            error!("Failed to remove enabled link: {}", e);
            return HttpResponse::InternalServerError().json(NginxResponse {
                success: false,
                message: format!("Enabled link silinə bilmədi: {}. Root icazəsi lazımdır.", e),
            });
        }
        info!("Symlink removed");
    }
    
    // Remove config file
    match fs::remove_file(&config_path) {
        Ok(_) => {
            info!("Config file removed");
            // Reload nginx
            let reload = std::process::Command::new("systemctl")
                .args(&["reload", "nginx"])
                .output();
            
            match reload {
                Ok(_) => {
                    info!("Nginx reloaded");
                    HttpResponse::Ok().json(NginxResponse {
                        success: true,
                        message: format!("✅ {} proxy konfiqurasiyası silindi", name.as_str()),
                    })
                },
                Err(e) => {
                    warn!("Failed to reload nginx: {}", e);
                    HttpResponse::Ok().json(NginxResponse {
                        success: true,
                        message: format!("✅ {} silindi (nginx reload edilmədi: {})", name.as_str(), e),
                    })
                }
            }
        },
        Err(e) => {
            error!("Failed to remove config file: {}", e);
            HttpResponse::InternalServerError().json(NginxResponse {
                success: false,
                message: format!("Silinə bilmədi: {}. Root icazəsi lazımdır.", e),
            })
        },
    }
}

/// Update an existing nginx proxy configuration
#[actix_web::put("/api/nginx/proxies/{name}")]
pub async fn update_nginx_proxy(data: web::Data<AppState>, name: web::Path<String>, proxy: web::Json<NginxProxy>) -> impl Responder {
    let proxy_name = name.into_inner();
    info!("PUT /api/nginx/proxies/{} - Updating proxy", proxy_name);

    let nginx_sites_available = &data.nginx_config.sites_available_path;
    let nginx_sites_enabled = &data.nginx_config.sites_enabled_path;

    // Validate that the name matches
    if proxy.name != proxy_name {
        return HttpResponse::BadRequest().json(NginxResponse {
            success: false,
            message: "URL-dəki ad və body-dəki ad uyğun gəlmir".to_string(),
        });
    }

    // Validate and format extra_config if provided
    let validated_proxy = if let Some(extra) = &proxy.extra_config {
        if !extra.trim().is_empty() {
            match validate_nginx_extra_config(extra) {
                Ok(formatted) => {
                    info!("Extra config validated and formatted");
                    NginxProxy {
                        name: proxy.name.clone(),
                        domain: proxy.domain.clone(),
                        backend: proxy.backend.clone(),
                        ssl: proxy.ssl,
                        extra_config: Some(formatted),
                    }
                },
                Err(e) => {
                    error!("Extra config validation failed: {}", e);
                    return HttpResponse::BadRequest().json(NginxResponse {
                        success: false,
                        message: format!("❌ Əlavə konfiqurasiya xətası: {}", e),
                    });
                }
            }
        } else {
            proxy.into_inner()
        }
    } else {
        proxy.into_inner()
    };

    let config_path = format!("{}/{}", nginx_sites_available, validated_proxy.name);
    let enabled_path = format!("{}/{}", nginx_sites_enabled, validated_proxy.name);
    let backup_path = format!("{}/{}.backup", nginx_sites_available, validated_proxy.name);

    // Check if config exists
    if !Path::new(&config_path).exists() {
        error!("Config file not found: {}", config_path);
        return HttpResponse::NotFound().json(NginxResponse {
            success: false,
            message: format!("Konfiqurasiya tapılmadı: {}", validated_proxy.name),
        });
    }

    // Backup existing config
    info!("Backing up existing config");
    if let Err(e) = fs::copy(&config_path, &backup_path) {
        error!("Failed to create backup: {}", e);
        return HttpResponse::InternalServerError().json(NginxResponse {
            success: false,
            message: format!("Yedək yaradıla bilmədi: {}", e),
        });
    }

    // Generate new config
    let config_content = generate_nginx_config(&validated_proxy);

    // Write new config
    match fs::write(&config_path, &config_content) {
        Ok(_) => {
            info!("Config file updated successfully");

            // Ensure symlink exists
            if !Path::new(&enabled_path).exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    if let Err(e) = symlink(&config_path, &enabled_path) {
                        error!("Failed to create symlink: {}", e);
                        let _ = fs::copy(&backup_path, &config_path);
                        let _ = fs::remove_file(&backup_path);
                        return HttpResponse::InternalServerError().json(NginxResponse {
                            success: false,
                            message: format!("Symlink yaradıla bilmədi: {}", e),
                        });
                    }
                }
            }

            // Test nginx config
            info!("Testing nginx configuration...");
            let output = std::process::Command::new("nginx")
                .args(&["-t"])
                .output();

            match output {
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    info!("Nginx test output: {}", stderr);

                    if result.status.success() {
                        // Test passed - remove backup and reload nginx
                        let _ = fs::remove_file(&backup_path);
                        info!("Backup removed after successful test");

                        info!("Reloading nginx...");
                        let reload = std::process::Command::new("systemctl")
                            .args(&["reload", "nginx"])
                            .output();

                        match reload {
                            Ok(reload_result) => {
                                if reload_result.status.success() {
                                    info!("Nginx reloaded successfully");
                                    HttpResponse::Ok().json(NginxResponse {
                                        success: true,
                                        message: format!("✅ {} proxy konfiqurasiyası yeniləndi", validated_proxy.domain),
                                    })
                                } else {
                                    let reload_err = String::from_utf8_lossy(&reload_result.stderr);
                                    error!("Nginx reload failed: {}", reload_err);
                                    HttpResponse::InternalServerError().json(NginxResponse {
                                        success: false,
                                        message: format!("Nginx reload edilə bilmədi: {}", reload_err),
                                    })
                                }
                            },
                            Err(e) => {
                                error!("Failed to execute systemctl: {}", e);
                                HttpResponse::InternalServerError().json(NginxResponse {
                                    success: false,
                                    message: format!("Systemctl çalışdırıla bilmədi: {}", e),
                                })
                            }
                        }
                    } else {
                        // Test failed - restore backup
                        error!("Nginx config test failed, rolling back");
                        info!("Restoring backup config");
                        match fs::copy(&backup_path, &config_path) {
                            Ok(_) => {
                                let _ = fs::remove_file(&backup_path);
                                info!("Backup restored successfully");
                                HttpResponse::BadRequest().json(NginxResponse {
                                    success: false,
                                    message: format!("❌ Nginx konfiqurasiya xətası: {}\n\n✅ Əvvəlki konfiqurasiya bərpa edildi.", stderr),
                                })
                            },
                            Err(e) => {
                                error!("Failed to restore backup: {}", e);
                                HttpResponse::InternalServerError().json(NginxResponse {
                                    success: false,
                                    message: format!("❌ Nginx xətası: {}\n❌ Yedək bərpa edilə bilmədi: {}", stderr, e),
                                })
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to test nginx config: {}", e);
                    let _ = fs::copy(&backup_path, &config_path);
                    let _ = fs::remove_file(&backup_path);
                    HttpResponse::InternalServerError().json(NginxResponse {
                        success: false,
                        message: format!("Nginx test edilə bilmədi: {}", e),
                    })
                }
            }
        },
        Err(e) => {
            error!("Failed to write config file: {}", e);
            let _ = fs::copy(&backup_path, &config_path);
            let _ = fs::remove_file(&backup_path);
            HttpResponse::InternalServerError().json(NginxResponse {
                success: false,
                message: format!("Fayl yazıla bilmədi: {}", e),
            })
        },
    }
}
