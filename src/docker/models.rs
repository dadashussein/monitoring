use serde::Serialize;

#[derive(Serialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
    pub created: i64,
    pub memory_usage: Option<u64>,
    pub memory_limit: Option<u64>,
    pub memory_percent: Option<f64>,
}

#[derive(Serialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: i64,
    pub created: i64,
}

#[derive(Serialize)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: Option<i64>,
}

#[derive(Serialize)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub subnet: Option<String>,
}

#[derive(Serialize)]
pub struct DockerResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct DockerLogsResponse {
    pub logs: String,
}
