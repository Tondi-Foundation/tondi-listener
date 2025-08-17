use std::time::Duration;

use tower_http::timeout::TimeoutLayer;

pub fn timeout(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::new(timeout)
}
