// src-tauri/src/auth.rs
use serde::{self, Deserialize, Deserializer, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

// Función auxiliar para deserializar campos que pueden ser String o número
pub fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumber;
    impl<'de> serde::de::Visitor<'de> for StringOrNumber {
        type Value = String;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a number")
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_owned())
        }
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }
    }
    deserializer.deserialize_any(StringOrNumber)
}

// Función auxiliar para deserializar campos opcionales que pueden ser String o número
pub fn string_or_number_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberOpt;
    impl<'de> serde::de::Visitor<'de> for StringOrNumberOpt {
        type Value = Option<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional string or number")
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            string_or_number(deserializer).map(Some)
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_owned()))
        }
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }
    }
    deserializer.deserialize_option(StringOrNumberOpt)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(deserialize_with = "string_or_number_opt")]
    pub id: Option<String>,
    pub nombre: Option<String>,
    pub email: String,
    pub suscripcion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub mensaje: Option<String>,
    pub token: String,
    pub usuario: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    error: String,
}

pub async fn login(email: String, password: String, server_url: &str) -> Result<AuthResponse, String> {
    let client = Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    // Usar la URL del servidor desde la configuración
    let login_url = format!("{}/api/auth/login", server_url);
    
    let login_request = LoginRequest {
        email,
        password,
    };
    
    match client.post(&login_url)
        .headers(headers)
        .json(&login_request)
        .send()
        .await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AuthResponse>().await {
                        Ok(auth_response) => Ok(auth_response),
                        Err(e) => Err(format!("Error al decodificar respuesta: {}", e)),
                    }
                } else {
                    // Intentar obtener mensaje de error de la API
                    // Formato exacto para ser compatible con el frontend existente
                    match response.json::<ApiError>().await {
                        Ok(api_error) => Err(api_error.error),
                        Err(_) => Err("Error de autenticación".to_string()),
                    }
                }
            },
            Err(e) => Err(format!("Error de conexión: {}", e)),
        }
}

// Función para verificar si un token es válido
pub async fn verify_token(token: &str, server_url: &str) -> Result<bool, String> {
    if token.is_empty() {
        return Ok(false);
    }
    
    // Para una verificación real, deberíamos llamar a un endpoint del servidor
    // Por ahora, solo devolvemos true si el token no está vacío
    Ok(true)
}