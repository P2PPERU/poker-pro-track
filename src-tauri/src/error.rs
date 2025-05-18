// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Error de red: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Error de serialización: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Error de autenticación: {0}")]
    Auth(String),
    
    #[error("Error en la detección de ventanas: {0}")]
    WindowDetection(String),
    
    #[error("Error en la configuración: {0}")]
    Config(String),
    
    #[error("Error en la API: {0}")]
    Api(String),
    
    #[error("Error desconocido: {0}")]
    Unknown(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}