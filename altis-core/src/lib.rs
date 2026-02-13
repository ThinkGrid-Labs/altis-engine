pub mod events;
pub mod search;
pub mod repository;
pub mod identity;
pub mod payment;
pub mod iata;
pub mod supplier;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Internal service error: {0}")]
    InternalError(String),
    #[error("Identity verification failed: {0}")]
    IdentityError(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

pub fn hello() {
    println!("Hello from Altis Core!");
}
