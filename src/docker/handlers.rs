// Docker management HTTP handlers

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use bollard::container::{
    ListContainersOptions, LogsOptions, RemoveContainerOptions, StatsOptions, StopContainerOptions,
};
use bollard::image::ListImagesOptions;
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use futures_util::stream::{StreamExt, TryStreamExt};
use log::{error, info};

use crate::docker::client::get_docker_client;
use crate::docker::models::{
    DockerContainer, DockerImage, DockerLogsResponse, DockerNetwork, DockerResponse, DockerVolume,
};

// HTML template for docker manager dashboard
const DOCKER_MANAGER_HTML: &str = include_str!("../templates/docker_manager.html");

/// Serve the docker manager HTML dashboard
#[get("/docker")]
pub async fn docker_manager() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DOCKER_MANAGER_HTML)
}

/// List all Docker containers with their status and resource usage
#[get("/api/docker/containers")]
pub async fn list_containers(data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("GET /api/docker/containers - Listing containers");

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
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
                let name = c
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let ports = c
                    .ports
                    .as_ref()
                    .map(|p| {
                        p.iter()
                            .filter_map(|port| match (port.public_port, port.private_port) {
                                (Some(pub_port), priv_port) => {
                                    Some(format!("{}:{}", pub_port, priv_port))
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                let container_id = c.id.as_ref().unwrap_or(&String::new()).clone();
                let state = c.state.as_ref().unwrap_or(&String::new()).clone();

                // Fetch memory stats for running containers
                let (memory_usage, memory_limit, memory_percent) = if state == "running" {
                    match docker
                        .stats(
                            &container_id,
                            Some(StatsOptions {
                                stream: false,
                                one_shot: true,
                            }),
                        )
                        .try_next()
                        .await
                    {
                        Ok(Some(stats)) => {
                            let mem_usage = stats.memory_stats.usage.unwrap_or(0);
                            let mem_limit = stats.memory_stats.limit.unwrap_or(0);
                            let mem_percent = if mem_limit > 0 {
                                (mem_usage as f64 / mem_limit as f64) * 100.0
                            } else {
                                0.0
                            };
                            (Some(mem_usage), Some(mem_limit), Some(mem_percent))
                        }
                        _ => (None, None, None),
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
        }
        Err(e) => {
            error!("Failed to list containers: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerContainer>)
        }
    }
}

/// Start a Docker container by ID
#[post("/api/docker/containers/{id}/start")]
pub async fn start_container(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("POST /api/docker/containers/{}/start", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    match docker.start_container::<String>(&id, None).await {
        Ok(_) => {
            info!("Container {} started", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container started successfully"),
            })
        }
        Err(e) => {
            error!("Failed to start container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to start container: {}", e),
            })
        }
    }
}

/// Stop a Docker container by ID
#[post("/api/docker/containers/{id}/stop")]
pub async fn stop_container(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("POST /api/docker/containers/{}/stop", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    let options = Some(StopContainerOptions { t: 10 });

    match docker.stop_container(&id, options).await {
        Ok(_) => {
            info!("Container {} stopped", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container stopped successfully"),
            })
        }
        Err(e) => {
            error!("Failed to stop container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to stop container: {}", e),
            })
        }
    }
}

/// Restart a Docker container by ID
#[post("/api/docker/containers/{id}/restart")]
pub async fn restart_container(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("POST /api/docker/containers/{}/restart", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    match docker.restart_container(&id, None).await {
        Ok(_) => {
            info!("Container {} restarted", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Container restarted successfully"),
            })
        }
        Err(e) => {
            error!("Failed to restart container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to restart container: {}", e),
            })
        }
    }
}

/// Remove a Docker container by ID
#[delete("/api/docker/containers/{id}")]
pub async fn remove_container(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("DELETE /api/docker/containers/{}", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
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
        }
        Err(e) => {
            error!("Failed to remove container: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove container: {}", e),
            })
        }
    }
}

/// Get logs from a Docker container by ID
#[get("/api/docker/containers/{id}/logs")]
pub async fn get_container_logs(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("GET /api/docker/containers/{}/logs", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerLogsResponse {
                logs: format!("Error: {}", e),
            })
        }
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
            }
            Err(e) => {
                error!("Error reading logs: {}", e);
                break;
            }
        }
    }

    HttpResponse::Ok().json(DockerLogsResponse { logs })
}

/// List all Docker images
#[get("/api/docker/images")]
pub async fn list_images(data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("GET /api/docker/images - Listing images");

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
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
            let result: Vec<DockerImage> = images
                .iter()
                .map(|img| {
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
                })
                .collect();

            info!("Found {} images", result.len());
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            error!("Failed to list images: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerImage>)
        }
    }
}

/// Remove a Docker image by ID
#[delete("/api/docker/images/{id}")]
pub async fn remove_image(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("DELETE /api/docker/images/{}", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    match docker.remove_image(&id, None, None).await {
        Ok(_) => {
            info!("Image {} removed", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Image removed successfully"),
            })
        }
        Err(e) => {
            error!("Failed to remove image: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove image: {}", e),
            })
        }
    }
}

/// List all Docker volumes
#[get("/api/docker/volumes")]
pub async fn list_volumes(data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("GET /api/docker/volumes - Listing volumes");

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
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
            let result: Vec<DockerVolume> = response
                .volumes
                .unwrap_or_default()
                .iter()
                .map(|vol| DockerVolume {
                    name: vol.name.clone(),
                    driver: vol.driver.clone(),
                    mountpoint: vol.mountpoint.clone(),
                    created: None,
                })
                .collect();

            info!("Found {} volumes", result.len());
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            error!("Failed to list volumes: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerVolume>)
        }
    }
}

/// Remove a Docker volume by name
#[delete("/api/docker/volumes/{name}")]
pub async fn remove_volume(name: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("DELETE /api/docker/volumes/{}", name);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    match docker.remove_volume(&name, None).await {
        Ok(_) => {
            info!("Volume {} removed", name);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Volume removed successfully"),
            })
        }
        Err(e) => {
            error!("Failed to remove volume: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove volume: {}", e),
            })
        }
    }
}

/// List all Docker networks
#[get("/api/docker/networks")]
pub async fn list_networks(data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("GET /api/docker/networks - Listing networks");

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
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
            let result: Vec<DockerNetwork> = networks
                .iter()
                .map(|net| {
                    let subnet = net
                        .ipam
                        .as_ref()
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
                })
                .collect();

            info!("Found {} networks", result.len());
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            error!("Failed to list networks: {}", e);
            HttpResponse::InternalServerError().json(vec![] as Vec<DockerNetwork>)
        }
    }
}

/// Remove a Docker network by ID
#[delete("/api/docker/networks/{id}")]
pub async fn remove_network(id: web::Path<String>, data: web::Data<crate::system::models::AppState>) -> impl Responder {
    info!("DELETE /api/docker/networks/{}", id);

    let docker = match get_docker_client(&data.docker_config.socket_path).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: e,
            })
        }
    };

    match docker.remove_network(&id).await {
        Ok(_) => {
            info!("Network {} removed", id);
            HttpResponse::Ok().json(DockerResponse {
                success: true,
                message: format!("✅ Network removed successfully"),
            })
        }
        Err(e) => {
            error!("Failed to remove network: {}", e);
            HttpResponse::InternalServerError().json(DockerResponse {
                success: false,
                message: format!("Failed to remove network: {}", e),
            })
        }
    }
}
