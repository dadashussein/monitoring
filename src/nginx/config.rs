use crate::nginx::models::NginxProxy;

/// Formats nginx configuration with proper indentation
pub fn format_nginx_config(config: &str) -> String {
    let mut formatted = String::new();
    let mut indent_level: i32 = 0;
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

/// Validates and formats nginx extra configuration
pub fn validate_nginx_extra_config(config: &str) -> Result<String, String> {
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

/// Generates complete nginx configuration for a proxy
pub fn generate_nginx_config(proxy: &NginxProxy) -> String {
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
