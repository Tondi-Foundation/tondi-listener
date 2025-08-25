pub mod codec;

#[allow(warnings)]
pub mod protowire {
    macro_rules! include_proto {
        ($package:tt) => {
            include!(concat!("../../../target/proto/", $package, ".rs"));
        };
    }

    include_proto!("pingpong");
    include_proto!("explorer");
}

pub use prost;
pub use tonic;
pub use tonic_web as web;
