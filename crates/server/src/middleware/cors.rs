use tower_http::cors::{Any, CorsLayer};
use crate::ctx::config::CorsConfig;

pub fn cors(config: &CorsConfig) -> CorsLayer {
    let mut cors = CorsLayer::new();
    
    // 设置允许的源
    if config.allowed_origins.is_empty() {
        cors = cors.allow_origin(Any);
    } else {
        for origin in &config.allowed_origins {
            cors = cors.allow_origin(origin.parse().unwrap_or_else(|_| Any));
        }
    }
    
    // 设置允许的方法
    if config.allowed_methods.is_empty() {
        cors = cors.allow_methods(Any);
    } else {
        for method in &config.allowed_methods {
            if let Ok(http_method) = method.parse() {
                cors = cors.allow_methods([http_method]);
            }
        }
    }
    
    // 设置允许的头部
    if config.allowed_headers.is_empty() {
        cors = cors.allow_headers(Any);
    } else {
        for header in &config.allowed_headers {
            if let Ok(http_header) = header.parse() {
                cors = cors.allow_headers([http_header]);
            }
        }
    }
    
    // 设置预检请求的缓存时间
    cors = cors.max_age(std::time::Duration::from_secs(config.max_age));
    
    // 允许凭据（cookies等）
    cors = cors.allow_credentials(true);
    
    cors
}

/// 生产环境的严格CORS配置
pub fn strict_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("https://yourdomain.com".parse::<http::HeaderValue>().unwrap())
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .max_age(std::time::Duration::from_secs(3600))
        .allow_credentials(false)
}
