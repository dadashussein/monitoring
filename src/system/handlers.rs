use actix_web::{delete, get, web, HttpResponse, Responder};
use sysinfo::{Disks, Networks, Pid, Signal, System};
use log::{info, warn};

use crate::utils::{bytes_to_gb, refresh_system};
use super::models::*;

// ==================== API Endpoints ====================

#[get("/")]
pub async fn index() -> impl Responder {
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
pub async fn get_system_info(data: web::Data<AppState>) -> impl Responder {
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
pub async fn get_cpu_info(data: web::Data<AppState>) -> impl Responder {
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
pub async fn get_cpu_usage(data: web::Data<AppState>) -> impl Responder {
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
pub async fn get_memory_info(data: web::Data<AppState>) -> impl Responder {
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
pub async fn get_disks_info() -> impl Responder {
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
pub async fn get_network_info() -> impl Responder {
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

#[get("/api/processes")]
pub async fn get_processes(data: web::Data<AppState>, query: web::Query<ProcessQuery>) -> impl Responder {
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
pub async fn get_load_average() -> impl Responder {
    let load = System::load_average();
    
    let load_avg = LoadAverage {
        one_min: load.one,
        five_min: load.five,
        fifteen_min: load.fifteen,
    };

    HttpResponse::Ok().json(load_avg)
}

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
    })
}

#[delete("/api/processes/{pid}")]
pub async fn kill_process(data: web::Data<AppState>, pid: web::Path<u32>) -> impl Responder {
    let mut system = data.system.lock().unwrap();
    system.refresh_processes();
    
    let pid_value = pid.into_inner();
    let process_pid = Pid::from_u32(pid_value);
    
    if let Some(process) = system.process(process_pid) {
        let process_name = process.name().to_string();
        
        // Try to kill the process
        if process.kill_with(Signal::Term).is_some() {
            info!("Process '{}' (PID: {}) terminated successfully", process_name, pid_value);
            HttpResponse::Ok().json(KillResponse {
                success: true,
                message: format!("Process '{}' (PID: {}) terminated successfully", process_name, pid_value),
                pid: pid_value,
            })
        } else {
            warn!("Failed to terminate process '{}' (PID: {})", process_name, pid_value);
            HttpResponse::InternalServerError().json(KillResponse {
                success: false,
                message: format!("Failed to terminate process '{}' (PID: {})", process_name, pid_value),
                pid: pid_value,
            })
        }
    } else {
        warn!("Process with PID {} not found", pid_value);
        HttpResponse::NotFound().json(KillResponse {
            success: false,
            message: format!("Process with PID {} not found", pid_value),
            pid: pid_value,
        })
    }
}

// Embedded dashboard HTML
const DASHBOARD_HTML: &str = include_str!("../templates/dashboard.html");

// Serve the dashboard HTML
#[get("/dashboard")]
pub async fn dashboard() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DASHBOARD_HTML)
}
