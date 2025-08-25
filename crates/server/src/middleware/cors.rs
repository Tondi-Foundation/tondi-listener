use tower_http::cors::{Any, CorsLayer};
use crate::ctx::config::CorsConfig;

pub fn cors(config: &CorsConfig) -> CorsLayer {
    let mut cors = CorsLayer::new();
    
    // Set allowed origins
    if config.allowed_origins.is_empty() {
        // If no configuration, allow all origins (equivalent to no CORS restrictions)
        cors = cors.allow_origin(Any);
    } else {
        for origin in &config.allowed_origins {
            if let Ok(header_value) = origin.parse::<http::HeaderValue>() {
                cors = cors.allow_origin(header_value);
            } else {
                cors = cors.allow_origin(Any);
            }
        }
    }
    
    // Set allowed methods
    if config.allowed_methods.is_empty() {
        // If no configuration, allow all methods
        cors = cors.allow_methods(Any);
    } else {
        for method in &config.allowed_methods {
            if let Ok(http_method) = method.parse() {
                cors = cors.allow_methods([http_method]);
            }
        }
    }
    
    // Set allowed headers
    if config.allowed_headers.is_empty() {
        // If no configuration, allow all headers
        cors = cors.allow_headers(Any);
    } else {
        for header in &config.allowed_headers {
            if let Ok(http_header) = header.parse() {
                cors = cors.allow_headers([http_header]);
            }
        }
    }
    
    // Set max age for preflight request caching
    cors = cors.max_age(std::time::Duration::from_secs(config.max_age));
    
    // Allow credentials (cookies, etc.)
    cors = cors.allow_credentials(true);
    
    cors
}

/// Fully open CORS configuration (equivalent to no CORS restrictions)
pub fn open_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(86400)) // 24 hours
}

/// Strict CORS configuration for production
pub fn strict_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("https://yourdomain.com".parse::<http::HeaderValue>().unwrap())
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .max_age(std::time::Duration::from_secs(3600))
        .allow_credentials(false)
}
