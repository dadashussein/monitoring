use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use sysinfo::{Disks, Networks, System, Pid, Signal};
use log::{info, warn, error};
use bollard::Docker;
use bollard::container::{ListContainersOptions, StopContainerOptions, RemoveContainerOptions, LogsOptions, StatsOptions};
use bollard::image::ListImagesOptions;
use bollard::volume::ListVolumesOptions;
use bollard::network::ListNetworksOptions;
use futures_util::stream::{StreamExt, TryStreamExt};

// Shared application state
struct AppState {
    system: Mutex<System>,
}

// ==================== Response Models ====================

#[derive(Serialize)]
struct SystemInfo {
    hostname: String,
    os_name: String,
    os_version: String,
    kernel_version: String,
    uptime_seconds: u64,
    boot_time: String,
}

#[derive(Serialize)]
struct CpuInfo {
    name: String,
    brand: String,
    physical_cores: usize,
    logical_cores: usize,
    frequency_mhz: u64,
    architecture: String,
}

#[derive(Serialize)]
struct CpuUsage {
    overall_usage_percent: f32,
    per_core_usage: Vec<f32>,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    available_bytes: u64,
    total_gb: f64,
    used_gb: f64,
    free_gb: f64,
    used_percent: f64,
}

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    mount_point: String,
    filesystem: String,
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
    total_gb: f64,
    used_gb: f64,
    available_gb: f64,
    used_percent: f64,
}

#[derive(Serialize)]
struct NetworkInfo {
    interface_name: String,
    mac_address: String,
    received_bytes: u64,
    transmitted_bytes: u64,
    received_packets: u64,
    transmitted_packets: u64,
    errors_in: u64,
    errors_out: u64,
}

#[derive(Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_bytes: u64,
    memory_mb: f64,
    parent_pid: Option<u32>,
    status: String,
    runtime_seconds: u64,
}

#[derive(Serialize)]
struct LoadAverage {
    one_min: f64,
    five_min: f64,
    fifteen_min: f64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

#[derive(Serialize)]
struct ApiInfo {
    name: String,
    version: String,
    description: String,
    endpoints: Vec<&'static str>,
}

// ==================== Helper Functions ====================

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

fn refresh_system(system: &mut System) {
    system.refresh_all();
}

// ==================== API Endpoints ====================

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(ApiInfo {
        name: "Ubuntu Resource API".to_string(),
        version: "0.1.0".to_string(),
        description: "REST API for monitoring Ubuntu system resources".to_string(),
        endpoints: vec![
            "/api/system",
            "/api/cpu",
            "/api/cpu/usage",
            "/api/memory",
            "/api/disks",
            "/api/network",
            "/api/processes",
            "/api/load",
            "/health",
        ],
    })
}

#[get("/api/system")]
async fn get_system_info(data: web::Data<AppState>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    refresh_system(&mut system);

    let info = SystemInfo {
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        uptime_seconds: System::uptime(),
        boot_time: chrono::DateTime::from_timestamp(System::boot_time() as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    };

    HttpResponse::Ok().json(info)
}

#[get("/api/cpu")]
async fn get_cpu_info(data: web::Data<AppState>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_cpu();

    let cpus = system.cpus();
    let first_cpu = cpus.first();

    let info = CpuInfo {
        name: first_cpu.map(|c| c.name().to_string()).unwrap_or_default(),
        brand: first_cpu.map(|c| c.brand().to_string()).unwrap_or_default(),
        physical_cores: system.physical_core_count().unwrap_or(0),
        logical_cores: cpus.len(),
        frequency_mhz: first_cpu.map(|c| c.frequency()).unwrap_or(0),
        architecture: std::env::consts::ARCH.to_string(),
    };

    HttpResponse::Ok().json(info)
}

#[get("/api/cpu/usage")]
async fn get_cpu_usage(data: web::Data<AppState>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_cpu_usage();
    
    // Wait a moment for CPU usage calculation
    std::thread::sleep(std::time::Duration::from_millis(200));
    system.refresh_cpu_usage();

    let cpus = system.cpus();
    let per_core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
    
    let overall_usage = if !per_core_usage.is_empty() {
        per_core_usage.iter().sum::<f32>() / per_core_usage.len() as f32
    } else {
        0.0
    };

    let usage = CpuUsage {
        overall_usage_percent: overall_usage,
        per_core_usage,
    };

    HttpResponse::Ok().json(usage)
}

#[get("/api/memory")]
async fn get_memory_info(data: web::Data<AppState>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_memory();

    let total = system.total_memory();
    let used = system.used_memory();
    let free = system.free_memory();
    let available = system.available_memory();

    let info = MemoryInfo {
        total_bytes: total,
        used_bytes: used,
        free_bytes: free,
        available_bytes: available,
        total_gb: bytes_to_gb(total),
        used_gb: bytes_to_gb(used),
        free_gb: bytes_to_gb(free),
        used_percent: if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 },
    };

    HttpResponse::Ok().json(info)
}

#[get("/api/disks")]
async fn get_disks_info() -> impl Responder {
    let disks = Disks::new_with_refreshed_list();
    
    let disk_info: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                filesystem: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                total_gb: bytes_to_gb(total),
                used_gb: bytes_to_gb(used),
                available_gb: bytes_to_gb(available),
                used_percent: if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 },
            }
        })
        .collect();

    HttpResponse::Ok().json(disk_info)
}

#[get("/api/network")]
async fn get_network_info() -> impl Responder {
    let networks = Networks::new_with_refreshed_list();
    
    let network_info: Vec<NetworkInfo> = networks
        .iter()
        .map(|(name, network)| NetworkInfo {
            interface_name: name.to_string(),
            mac_address: network.mac_address().to_string(),
            received_bytes: network.total_received(),
            transmitted_bytes: network.total_transmitted(),
            received_packets: network.total_packets_received(),
            transmitted_packets: network.total_packets_transmitted(),
            errors_in: network.total_errors_on_received(),
            errors_out: network.total_errors_on_transmitted(),
        })
        .collect();

    HttpResponse::Ok().json(network_info)
}

#[derive(Deserialize)]
struct ProcessQuery {
    limit: Option<usize>,
}

#[get("/api/processes")]
async fn get_processes(data: web::Data<AppState>, query: web::Query<ProcessQuery>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_processes();

    let limit = query.limit.unwrap_or(50);

    let mut processes: Vec<ProcessInfo> = system
        .processes()
        .values()
        .map(|process| ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_bytes: process.memory(),
            memory_mb: process.memory() as f64 / (1024.0 * 1024.0),
            parent_pid: process.parent().map(|p| p.as_u32()),
            status: format!("{:?}", process.status()),
            runtime_seconds: process.run_time(),
        })
        .collect();

    // Sort by CPU usage descending
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
    processes.truncate(limit);

    HttpResponse::Ok().json(processes)
}

#[get("/api/load")]
async fn get_load_average() -> impl Responder {
    let load = System::load_average();
    
    let load_avg = LoadAverage {
        one_min: load.one,
        five_min: load.five,
        fifteen_min: load.fifteen,
    };

    HttpResponse::Ok().json(load_avg)
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
    })
}

#[derive(Serialize)]
struct KillResponse {
    success: bool,
    message: String,
    pid: u32,
}

#[delete("/api/processes/{pid}")]
async fn kill_process(data: web::Data<AppState>, pid: web::Path<u32>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_processes();
    
    let pid_value = pid.into_inner();
    let process_pid = Pid::from_u32(pid_value);
    
    if let Some(process) = system.process(process_pid) {
        let process_name = process.name().to_string();
        
        // Try to kill the process
        if process.kill_with(Signal::Term).is_some() {
            HttpResponse::Ok().json(KillResponse {
                success: true,
                message: format!("Process '{}' (PID: {}) terminated successfully", process_name, pid_value),
                pid: pid_value,
            })
        } else {
            HttpResponse::InternalServerError().json(KillResponse {
                success: false,
                message: format!("Failed to terminate process '{}' (PID: {})", process_name, pid_value),
                pid: pid_value,
            })
        }
    } else {
        HttpResponse::NotFound().json(KillResponse {
            success: false,
            message: format!("Process with PID {} not found", pid_value),
            pid: pid_value,
        })
    }
}

// Embedded dashboard HTML
const DASHBOARD_HTML: &str = include_str!("dashboard.html");
const NGINX_ADMIN_HTML: &str = include_str!("nginx_admin.html");
const DOCKER_MANAGER_HTML: &str = include_str!("docker_manager.html");

// Serve the dashboard HTML
#[get("/dashboard")]
async fn dashboard() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DASHBOARD_HTML)
}

// Serve the nginx admin HTML
#[get("/nginx")]
async fn nginx_admin() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(NGINX_ADMIN_HTML)
}

// Serve the docker manager HTML
#[get("/docker")]
async fn docker_manager() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DOCKER_MANAGER_HTML)
}

// ==================== Nginx Proxy Management ====================

#[derive(Serialize, Deserialize, Clone)]
struct NginxProxy {
    name: String,
    domain: String,
    backend: String,
    ssl: bool,
    extra_config: Option<String>,
}

#[derive(Serialize)]
struct NginxResponse {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct FormatRequest {
    config: String,
}

#[derive(Serialize)]
struct FormatResponse {
    success: bool,
    formatted: Option<String>,
    error: Option<String>,
}

const NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";

fn format_nginx_config(config: &str) -> String {
    let mut formatted = String::new();
    let mut indent_level = 0;
    let indent = "    "; // 4 spaces
    
    for line in config.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }
        
        // Decrease indent for closing braces
        if trimmed.starts_with('}') {
            indent_level = indent_level.saturating_sub(1);
        }
        
        // Add indentation
        for _ in 0..indent_level {
            formatted.push_str(indent);
        }
        
        // Add the line
        formatted.push_str(trimmed);
        formatted.push('\n');
        
        // Increase indent for opening braces
        if trimmed.ends_with('{') {
            indent_level += 1;
        }
    }
    
    formatted.trim_end().to_string()
}

fn validate_nginx_extra_config(config: &str) -> Result<String, String> {
    // Format the config first
    let formatted = format_nginx_config(config);
    
    // Basic syntax validation
    let open_braces = formatted.matches('{').count();
    let close_braces = formatted.matches('}').count();
    
    if open_braces != close_braces {
        return Err(format!("Sintaksis xətası: Açılan və bağlanan mötərizələr uyğun gəlmir (açıq: {}, bağlı: {})", open_braces, close_braces));
    }
    
    // Check for common mistakes
    for (i, line) in formatted.lines().enumerate() {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        
        // Check if directives end with semicolon (except blocks)
        if !trimmed.ends_with('{') && !trimmed.ends_with('}') && !trimmed.ends_with(';') {
            // Check if it's a location or server block start
            if !trimmed.starts_with("location") && !trimmed.starts_with("server") && !trimmed.starts_with("if") {
                return Err(format!("Sətir {}: Direktiv nöqtəli vergüllə (;) bitməlidir: {}", i + 1, trimmed));
            }
        }
    }
    
    Ok(formatted)
}

#[post("/api/nginx/format")]
async fn format_nginx_extra_config(req: web::Json<FormatRequest>) -> impl Responder {
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

fn generate_nginx_config(proxy: &NginxProxy) -> String {
    let ssl_config = if proxy.ssl {
        format!(r#"
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    
    # SSL sertifikatları (Let's Encrypt və ya özəl)
    # ssl_certificate /etc/letsencrypt/live/{}/fullchain.pem;
    # ssl_certificate_key /etc/letsencrypt/live/{}/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;"#, proxy.domain, proxy.domain)
    } else {
        "    listen 80;\n    listen [::]:80;".to_string()
    };

    // Process extra config - check if it contains location blocks
    let (extra_in_location, extra_in_server) = if let Some(extra) = &proxy.extra_config {
        let trimmed = extra.trim();
        if trimmed.contains("location") {
            // If it has location blocks, add them at server level
            (String::new(), format!("\n    # Əlavə konfiqurasiya\n{}", 
                trimmed.lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n")))
        } else {
            // Otherwise add directives inside location /
            (format!("\n        # Əlavə konfiqurasiya\n{}", 
                trimmed.lines()
                    .map(|line| format!("        {}", line))
                    .collect::<Vec<_>>()
                    .join("\n")), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    format!(r#"# Nginx Reverse Proxy - {}
# Yaradılma: {}
# Backend: {}

server {{
{}
    server_name {};

    location / {{
        proxy_pass {};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;{}
    }}{}
}}
"#, proxy.name, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), proxy.backend, ssl_config, proxy.domain, proxy.backend, extra_in_location, extra_in_server)
}

#[get("/api/nginx/proxies")]
async fn get_nginx_proxies() -> impl Responder {
    info!("GET /api/nginx/proxies - Listing nginx configurations");
    
    let mut proxies = Vec::new();
    
    // Check if directory exists
    if !Path::new(NGINX_SITES_AVAILABLE).exists() {
        warn!("Nginx sites-available directory not found: {}", NGINX_SITES_AVAILABLE);
        return HttpResponse::Ok().json(serde_json::json!({
            "proxies": proxies,
            "warning": format!("Nginx konfiqurasiya qovluğu tapılmadı: {}. Nginx quraşdırılıb?", NGINX_SITES_AVAILABLE)
        }));
    }
    
    match fs::read_dir(NGINX_SITES_AVAILABLE) {
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

#[post("/api/nginx/proxies")]
async fn create_nginx_proxy(proxy: web::Json<NginxProxy>) -> impl Responder {
    info!("POST /api/nginx/proxies - Creating proxy: {} -> {}", proxy.domain, proxy.backend);
    
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
    if !Path::new(NGINX_SITES_AVAILABLE).exists() {
        error!("Nginx sites-available directory not found");
        return HttpResponse::InternalServerError().json(NginxResponse {
            success: false,
            message: format!("Nginx qovluğu tapılmadı: {}. Nginx quraşdırılıb?", NGINX_SITES_AVAILABLE),
        });
    }
    
    let config_path = format!("{}/{}", NGINX_SITES_AVAILABLE, validated_proxy.name);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, validated_proxy.name);
    let backup_path = format!("{}/{}.backup", NGINX_SITES_AVAILABLE, validated_proxy.name);
    
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
                                        message: format!("✅ {} proxy konfiqurasiyası yaradıldı və aktiv edildi", proxy.domain),
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

#[delete("/api/nginx/proxies/{name}")]
async fn delete_nginx_proxy(name: web::Path<String>) -> impl Responder {
    info!("DELETE /api/nginx/proxies/{} - Deleting proxy", name);
    
    let config_path = format!("{}/{}", NGINX_SITES_AVAILABLE, name.as_str());
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, name.as_str());
    
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

// ==================== Docker Management ====================

#[derive(Serialize)]
struct DockerContainer {
    id: String,
    name: String,
    image: String,
    state: String,
    status: String,
    ports: String,
    created: i64,
    memory_usage: Option<u64>,
    memory_limit: Option<u64>,
    memory_percent: Option<f64>,
}

#[derive(Serialize)]
struct DockerImage {
    id: String,
    repository: String,
    tag: String,
    size: i64,
    created: i64,
}

#[derive(Serialize)]
struct DockerVolume {
    name: String,
    driver: String,
    mountpoint: String,
    created: Option<i64>,
}

#[derive(Serialize)]
struct DockerNetwork {
    id: String,
    name: String,
    driver: String,
    scope: String,
    subnet: Option<String>,
}

#[derive(Serialize)]
struct DockerResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct DockerLogsResponse {
    logs: String,
}

async fn get_docker_client() -> Result<Docker, String> {
    Docker::connect_with_socket_defaults()
        .map_err(|e| format!("Docker connection failed: {}. Is Docker running?", e))
}

#[get("/api/docker/containers")]
async fn list_containers() -> impl Responder {
    info!("GET /api/docker/containers - Listing containers");
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().json(vec![] as Vec<DockerContainer>);
        }
    };
    
    let options = Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });
    
    match docker.list_containers(options).await {
        Ok(containers) => {
            let mut result: Vec<DockerContainer> = Vec::new();
            
            for c in containers.iter() {
                let name = c.names.as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let ports = c.ports.as_ref()
                    .map(|p| p.iter()
                        .filter_map(|port| {
                            match (port.public_port, port.private_port) {
                                (Some(pub_port), priv_port) => Some(format!("{}:{}", pub_port, priv_port)),
                                _ => None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default();
                
                let container_id = c.id.as_ref().unwrap_or(&String::new()).clone();
                let state = c.state.as_ref().unwrap_or(&String::new()).clone();
                
                // Fetch memory stats for running containers
                let (memory_usage, memory_limit, memory_percent) = if state == "running" {
                    match docker.stats(&container_id, Some(StatsOptions { stream: false, one_shot: true })).try_next().await {
                        Ok(Some(stats)) => {
                            let mem_usage = stats.memory_stats.usage.unwrap_or(0);
                            let mem_limit = stats.memory_stats.limit.unwrap_or(0);
                            let mem_percent = if mem_limit > 0 {
                                (mem_usage as f64 / mem_limit as f64) * 100.0
                            } else {
                                0.0
                            };
                            (Some(mem_usage), Some(mem_limit), Some(mem_percent))
                        },
                        _ => (None, None, None)
                    }
                } else {
                    (None, None, None)
                };
                
                result.push(DockerContainer {
                    id: container_id,
                    name,
                    image: c.image.as_ref().unwrap_or(&String::new()).clone(),
                    state,
                    status: c.status.as_ref().unwrap_or(&String::new()).clone(),
                    ports,
                    created: c.created.unwrap_or(0),
                    memory_usage,
                    memory_limit,
                    memory_percent,
                });
            }
            
            info!("Found {} containers", result.len());
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            error!("Failed to list containers: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerContainer>)
        }
    }
}

#[post("/api/docker/containers/{id}/start")]
async fn start_container(id: web::Path<String>) -> impl Responder {
    info!("POST /api/docker/containers/{}/start", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    match docker.start_container::<String>(&id, None).await {
        Ok(_) => {
            info!("Container {} started", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container started successfully"),
            })
        },
        Err(e) => {
            error!("Failed to start container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to start container: {}", e),
            })
        }
    }
}

#[post("/api/docker/containers/{id}/stop")]
async fn stop_container(id: web::Path<String>) -> impl Responder {
    info!("POST /api/docker/containers/{}/stop", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    let options = Some(StopContainerOptions { t: 10 });
    
    match docker.stop_container(&id, options).await {
        Ok(_) => {
            info!("Container {} stopped", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container stopped successfully"),
            })
        },
        Err(e) => {
            error!("Failed to stop container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to stop container: {}", e),
            })
        }
    }
}

#[post("/api/docker/containers/{id}/restart")]
async fn restart_container(id: web::Path<String>) -> impl Responder {
    info!("POST /api/docker/containers/{}/restart", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    match docker.restart_container(&id, None).await {
        Ok(_) => {
            info!("Container {} restarted", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container restarted successfully"),
            })
        },
        Err(e) => {
            error!("Failed to restart container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to restart container: {}", e),
            })
        }
    }
}

#[delete("/api/docker/containers/{id}")]
async fn remove_container(id: web::Path<String>) -> impl Responder {
    info!("DELETE /api/docker/containers/{}", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    let options = Some(RemoveContainerOptions {
        force: true,
        ..Default::default()
    });
    
    match docker.remove_container(&id, options).await {
        Ok(_) => {
            info!("Container {} removed", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container removed successfully"),
            })
        },
        Err(e) => {
            error!("Failed to remove container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove container: {}", e),
            })
        }
    }
}

#[get("/api/docker/containers/{id}/logs")]
async fn get_container_logs(id: web::Path<String>) -> impl Responder {
    info!("GET /api/docker/containers/{}/logs", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerLogsResponse {
            logs: format!("Error: {}", e),
        }),
    };
    
    let options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: "100".to_string(),
        ..Default::default()
    });
    
    let mut logs = String::new();
    let mut stream = docker.logs(&id, options);
    
    while let Some(log) = stream.next().await {
        match log {
            Ok(output) => {
                logs.push_str(&output.to_string());
            },
            Err(e) => {
                error!("Error reading logs: {}", e);
                break;
            }
        }
    }
    
    HttpResponse::Ok().json(DockerLogsResponse { logs })
}

#[get("/api/docker/images")]
async fn list_images() -> impl Responder {
    info!("GET /api/docker/images - Listing images");
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().json(vec![] as Vec<DockerImage>);
        }
    };
    
    let options = Some(ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    });
    
    match docker.list_images(options).await {
        Ok(images) => {
            let result: Vec<DockerImage> = images.iter().map(|img| {
                let default_tag = "<none>:<none>".to_string();
                
                // Get first repo tag or use default
                let repo_tag_str = if !img.repo_tags.is_empty() {
                    img.repo_tags[0].clone()
                } else {
                    default_tag
                };
                
                let parts: Vec<&str> = repo_tag_str.split(':').collect();
                let repository = parts.get(0).unwrap_or(&"<none>").to_string();
                let tag = parts.get(1).unwrap_or(&"<none>").to_string();
                
                DockerImage {
                    id: img.id.clone(),
                    repository,
                    tag,
                    size: img.size,
                    created: img.created,
                }
            }).collect();
            
            info!("Found {} images", result.len());
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            error!("Failed to list images: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerImage>)
        }
    }
}

#[delete("/api/docker/images/{id}")]
async fn remove_image(id: web::Path<String>) -> impl Responder {
    info!("DELETE /api/docker/images/{}", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    match docker.remove_image(&id, None, None).await {
        Ok(_) => {
            info!("Image {} removed", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Image removed successfully"),
            })
        },
        Err(e) => {
            error!("Failed to remove image: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove image: {}", e),
            })
        }
    }
}

#[get("/api/docker/volumes")]
async fn list_volumes() -> impl Responder {
    info!("GET /api/docker/volumes - Listing volumes");
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().json(vec![] as Vec<DockerVolume>);
        }
    };
    
    let options = ListVolumesOptions::<String> {
        ..Default::default()
    };
    
    match docker.list_volumes(Some(options)).await {
        Ok(response) => {
            let result: Vec<DockerVolume> = response.volumes.unwrap_or_default().iter().map(|vol| {
                DockerVolume {
                    name: vol.name.clone(),
                    driver: vol.driver.clone(),
                    mountpoint: vol.mountpoint.clone(),
                    created: None,
                }
            }).collect();
            
            info!("Found {} volumes", result.len());
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            error!("Failed to list volumes: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerVolume>)
        }
    }
}

#[delete("/api/docker/volumes/{name}")]
async fn remove_volume(name: web::Path<String>) -> impl Responder {
    info!("DELETE /api/docker/volumes/{}", name);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    match docker.remove_volume(&name, None).await {
        Ok(_) => {
            info!("Volume {} removed", name);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Volume removed successfully"),
            })
        },
        Err(e) => {
            error!("Failed to remove volume: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove volume: {}", e),
            })
        }
    }
}

#[get("/api/docker/networks")]
async fn list_networks() -> impl Responder {
    info!("GET /api/docker/networks - Listing networks");
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().json(vec![] as Vec<DockerNetwork>);
        }
    };
    
    let options = ListNetworksOptions::<String> {
        ..Default::default()
    };
    
    match docker.list_networks(Some(options)).await {
        Ok(networks) => {
            let result: Vec<DockerNetwork> = networks.iter().map(|net| {
                let subnet = net.ipam.as_ref()
                    .and_then(|ipam| ipam.config.as_ref())
                    .and_then(|configs| configs.first())
                    .and_then(|config| config.subnet.clone());
                
                DockerNetwork {
                    id: net.id.as_ref().unwrap_or(&String::new()).clone(),
                    name: net.name.as_ref().unwrap_or(&String::new()).clone(),
                    driver: net.driver.as_ref().unwrap_or(&String::new()).clone(),
                    scope: net.scope.as_ref().unwrap_or(&String::new()).clone(),
                    subnet,
                }
            }).collect();
            
            info!("Found {} networks", result.len());
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            error!("Failed to list networks: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerNetwork>)
        }
    }
}

#[delete("/api/docker/networks/{id}")]
async fn remove_network(id: web::Path<String>) -> impl Responder {
    info!("DELETE /api/docker/networks/{}", id);
    
    let docker = match get_docker_client().await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(DockerResponse {
            success: false,
            message: e,
        }),
    };
    
    match docker.remove_network(&id).await {
        Ok(_) => {
            info!("Network {} removed", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Network removed successfully"),
            })
        },
        Err(e) => {
            error!("Failed to remove network: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove network: {}", e),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Get address and port from command-line arguments or use defaults
    let args: Vec<String> = std::env::args().collect();
    let address = args.get(1).map(|s| s.as_str()).unwrap_or("10.0.0.1");
    let port = args.get(2).map(|s| s.as_str()).unwrap_or("3012");
    let bind_addr = format!("{}:{}", address, port);

    let system = System::new_all();

    let app_state = web::Data::new(AppState {
        system: Mutex::new(system),
    });

    info!("🚀 Ubuntu Resource API starting on http://{}", bind_addr);
    println!("🚀 Ubuntu Resource API starting on http://{}", bind_addr);
    println!("📊 Dashboard: http://{}/dashboard", bind_addr);
    println!("🔄 Nginx Manager: http://{}/nginx", bind_addr);
    println!("🐳 Docker Manager: http://{}/docker", bind_addr);
    println!("");
    println!("📡 Available endpoints:");
    println!("   GET    /           - API info");
    println!("   GET    /dashboard  - Web dashboard");
    println!("   GET    /nginx      - Nginx proxy manager");
    println!("   GET    /docker     - Docker container manager");
    println!("   GET    /api/system - System information");
    println!("   GET    /api/cpu    - CPU information");
    println!("   GET    /api/cpu/usage - CPU usage statistics");
    println!("   GET    /api/memory - Memory information");
    println!("   GET    /api/disks  - Disk information");
    println!("   GET    /api/network - Network interfaces");
    println!("   GET    /api/processes - Running processes (?limit=N)");
    println!("   GET    /api/load   - System load average");
    println!("   GET    /health     - Health check");
    println!("   DELETE /api/processes/:pid - Kill process by PID");
    println!("   GET    /api/nginx/proxies - List nginx proxies");
    println!("   POST   /api/nginx/proxies - Create nginx proxy");
    println!("   POST   /api/nginx/format - Format and validate nginx config");
    println!("   DELETE /api/nginx/proxies/:name - Delete nginx proxy");
    println!("   GET    /api/docker/containers - List containers");
    println!("   POST   /api/docker/containers/:id/start - Start container");
    println!("   POST   /api/docker/containers/:id/stop - Stop container");
    println!("   POST   /api/docker/containers/:id/restart - Restart container");
    println!("   DELETE /api/docker/containers/:id - Remove container");
    println!("   GET    /api/docker/containers/:id/logs - Get container logs");
    println!("   GET    /api/docker/images - List images");
    println!("   DELETE /api/docker/images/:id - Remove image");
    println!("   GET    /api/docker/volumes - List volumes");
    println!("   DELETE /api/docker/volumes/:name - Remove volume");
    println!("   GET    /api/docker/networks - List networks");
    println!("   DELETE /api/docker/networks/:id - Remove network");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .service(index)
            .service(dashboard)
            .service(nginx_admin)
            .service(docker_manager)
            .service(get_system_info)
            .service(get_cpu_info)
            .service(get_cpu_usage)
            .service(get_memory_info)
            .service(get_disks_info)
            .service(get_network_info)
            .service(get_processes)
            .service(get_load_average)
            .service(health_check)
            .service(kill_process)
            .service(get_nginx_proxies)
            .service(create_nginx_proxy)
            .service(delete_nginx_proxy)
            .service(format_nginx_extra_config)
            .service(list_containers)
            .service(start_container)
            .service(stop_container)
            .service(restart_container)
            .service(remove_container)
            .service(get_container_logs)
            .service(list_images)
            .service(remove_image)
            .service(list_volumes)
            .service(remove_volume)
            .service(list_networks)
            .service(remove_network)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
