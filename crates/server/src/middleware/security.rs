use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use tokio::sync::RwLock;
use tower::{Layer, Service};

/// Rate limiter - 速率限制器
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub async fn is_allowed(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        // Clean up expired request records
        if let Some(timestamps) = requests.get_mut(key) {
            timestamps.retain(|&timestamp| now.duration_since(timestamp) < self.window);
            
            if timestamps.len() < self.max_requests as usize {
                timestamps.push(now);
                true
            } else {
                false
            }
        } else {
            requests.insert(key.to_string(), vec![now]);
            true
        }
    }
}

/// Rate limit middleware - 速率限制中间件
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    rate_limiter: RateLimiter,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            rate_limiter: RateLimiter::new(max_requests, window),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimitService {
            inner: service,
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitService<S> {
    inner: S,
    rate_limiter: RateLimiter,
}

impl<S> Service<Request> for RateLimitService<S>
where
    S: Service<Request> + Clone + Send + Sync,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let rate_limiter = self.rate_limiter.clone();
        
        Box::pin(async move {
            // Use a global rate limiter instead of IP-based
            if !rate_limiter.is_allowed("global").await {
                let response = (
                    http::StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": {
                            "code": "RATE_LIMIT_EXCEEDED",
                            "message": "Request too frequent, please try again later",
                            "retry_after": 60
                        }
                    }))
                ).into_response();
                
                return Ok(response);
            }

            inner.call(req).await
        })
    }
}

/// Create rate limit middleware - 创建速率限制中间件
pub fn rate_limit(max_requests: u32) -> RateLimitLayer {
    RateLimitLayer::new(max_requests, Duration::from_secs(60))
}

/// Request validation middleware - 请求验证中间件
#[derive(Debug, Clone)]
pub struct RequestValidationLayer;

impl<S> Layer<S> for RequestValidationLayer {
    type Service = RequestValidationService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestValidationService { inner: service }
    }
}

#[derive(Debug, Clone)]
pub struct RequestValidationService<S> {
    inner: S,
}

impl<S> Service<Request> for RequestValidationService<S>
where
    S: Service<Request> + Clone + Send + Sync,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // Validate request headers
            if let Some(user_agent) = req.headers().get("user-agent") {
                if user_agent.as_bytes().len() > 1024 {
                    let response = (
                        http::StatusCode::BAD_REQUEST,
                        axum::Json(serde_json::json!({
                            "error": {
                                "code": "INVALID_USER_AGENT",
                                "message": "Invalid User-Agent header"
                            }
                        }))
                    ).into_response();
                    
                    return Ok(response);
                }
            }

            // Validate content type for POST requests
            if req.method() == http::Method::POST {
                if let Some(content_type) = req.headers().get("content-type") {
                    let content_type_str = content_type.to_str().unwrap_or("");
                    if !content_type_str.starts_with("application/json") && 
                       !content_type_str.starts_with("text/plain") &&
                       !content_type_str.starts_with("multipart/form-data") {
                        let response = (
                            http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                            axum::Json(serde_json::json!({
                                "error": {
                                    "code": "UNSUPPORTED_MEDIA_TYPE",
                                    "message": "Unsupported media type"
                                }
                            }))
                        ).into_response();
                        
                        return Ok(response);
                    }
                }
            }

            inner.call(req).await
        })
    }
}

/// Create request validation middleware - 创建请求验证中间件
pub fn request_validation() -> RequestValidationLayer {
    RequestValidationLayer
}
