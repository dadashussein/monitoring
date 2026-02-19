use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::System;
use crate::config::{NginxConfig, DockerConfig};

// Shared application state
pub struct AppState {
    pub system: Mutex<System>,
    pub nginx_config: NginxConfig,
    pub docker_config: DockerConfig,
}

// ==================== Response Models ====================

#[derive(Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub boot_time: String,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub brand: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub frequency_mhz: u64,
    pub architecture: String,
}

#[derive(Serialize)]
pub struct CpuUsage {
    pub overall_usage_percent: f32,
    pub per_core_usage: Vec<f32>,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub used_percent: f64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub used_percent: f64,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub mac_address: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub received_packets: u64,
    pub transmitted_packets: u64,
    pub errors_in: u64,
    pub errors_out: u64,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub memory_mb: f64,
    pub parent_pid: Option<u32>,
    pub status: String,
    pub runtime_seconds: u64,
}

#[derive(Serialize)]
pub struct LoadAverage {
    pub one_min: f64,
    pub five_min: f64,
    pub fifteen_min: f64,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<&'static str>,
}

#[derive(Deserialize)]
pub struct ProcessQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct KillResponse {
    pub success: bool,
    pub message: String,
    pub pid: u32,
}
