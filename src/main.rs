use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use sysinfo::{Disks, Networks, System, Pid, Signal};
use log::{info, warn, error};

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

const NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";

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

    let extra = proxy.extra_config.as_ref()
        .map(|e| format!("\n    # Əlavə konfiqurasiya\n    {}", e))
        .unwrap_or_default();

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
    }}
}}
"#, proxy.name, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), proxy.backend, ssl_config, proxy.domain, proxy.backend, extra)
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
    
    // Check if nginx directories exist
    if !Path::new(NGINX_SITES_AVAILABLE).exists() {
        error!("Nginx sites-available directory not found");
        return HttpResponse::InternalServerError().json(NginxResponse {
            success: false,
            message: format!("Nginx qovluğu tapılmadı: {}. Nginx quraşdırılıb?", NGINX_SITES_AVAILABLE),
        });
    }
    
    let config_path = format!("{}/{}", NGINX_SITES_AVAILABLE, proxy.name);
    let enabled_path = format!("{}/{}", NGINX_SITES_ENABLED, proxy.name);
    
    info!("Config path: {}", config_path);
    info!("Enabled path: {}", enabled_path);
    
    // Generate nginx config
    let config_content = generate_nginx_config(&proxy);
    
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
                        // Reload nginx
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
                        // Rollback on error
                        error!("Nginx config test failed, rolling back");
                        let _ = fs::remove_file(&config_path);
                        let _ = fs::remove_file(&enabled_path);
                        HttpResponse::BadRequest().json(NginxResponse {
                            success: false,
                            message: format!("Nginx konfiqurasiya xətası: {}", stderr),
                        })
                    }
                },
                Err(e) => {
                    error!("Failed to test nginx config: {}", e);
                    HttpResponse::InternalServerError().json(NginxResponse {
                        success: false,
                        message: format!("Nginx test edilə bilmədi: {}. Nginx quraşdırılıb?", e),
                    })
                }
            }
        },
        Err(e) => {
            error!("Failed to write config file: {}", e);
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
    println!("");
    println!("📡 Available endpoints:");
    println!("   GET    /           - API info");
    println!("   GET    /dashboard  - Web dashboard");
    println!("   GET    /nginx      - Nginx proxy manager");
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
    println!("   DELETE /api/nginx/proxies/:name - Delete nginx proxy");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .service(index)
            .service(dashboard)
            .service(nginx_admin)
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
    })
    .bind(&bind_addr)?
    .run()
    .await
}
