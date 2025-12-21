// Payment integration module
// Handles Paddle payment provider integration

pub mod config;
pub mod paddle;
pub mod types;

pub use config::PaddleConfig;
pub use paddle::PaddleClient;
pub use types::*;

