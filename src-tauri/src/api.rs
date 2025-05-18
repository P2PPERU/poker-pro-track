// src-tauri/src/api.rs
use serde::{Deserialize, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};

// Estructura para las estadísticas del jugador
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerStats {
    pub player_name: String,
    pub vpip: String,
    pub pfr: String,
    pub three_bet: String,
    pub fold_to_3bet_pct: String,
    pub wtsd: String,
    pub wsd: String,
    pub cbet_flop: String,
    pub cbet_turn: String,
    pub fold_to_flop_cbet_pct: String,
    pub fold_to_turn_cbet_pct: String,
    pub limp_pct: Option<String>,
    pub limp_raise_pct: Option<String>,
    pub four_bet_preflop_pct: Option<String>,
    pub fold_to_4bet_pct: Option<String>,
    pub probe_bet_turn_pct: Option<String>,
    pub bet_river_pct: Option<String>,
    pub fold_to_river_bet_pct: Option<String>,
    pub overbet_turn_pct: Option<String>,
    pub overbet_river_pct: Option<String>,
    pub wsdwbr_pct: Option<String>,
    pub wwsf: String,
    pub total_manos: String,
    pub bb_100: String,
    pub win_usd: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    error: String,
}

// Función para obtener estadísticas del jugador
pub async fn get_player_stats(nick: String, sala: String, token: String, server_url: String) -> Result<PlayerStats, String> {
    let client = Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    // Añadir token de autenticación si está disponible
    if !token.is_empty() {
        let auth_value = format!("Bearer {}", token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    }
    
    // Codificar el nickname para URL
    let nick_encoded = urlencoding::encode(&nick);
    let url = format!("{}/api/jugador/{}/{}", server_url, sala, nick_encoded);
    
    match client.get(&url)
        .headers(headers)
        .send()
        .await {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.map_err(|e| format!("Error leyendo cuerpo: {}", e))?;
                
                if status.is_success() {
                    match serde_json::from_str::<PlayerStats>(&text) {
                        Ok(stats) => Ok(stats),
                        Err(e) => Err(format!("Error al decodificar respuesta: {}", e)),
                    }
                } else {
                    // Intentar obtener mensaje de error de la API
                    match serde_json::from_str::<ApiError>(&text) {
                        Ok(api_error) => Err(api_error.error),
                        Err(_) => Err(format!("Error en la API: {}", status)),
                    }
                }
            },
            Err(e) => Err(format!("Error de conexión: {}", e)),
        }
}

// Función para analizar estadísticas usando GPT
pub async fn analyze_stats(data: PlayerStats, api_key: String) -> Result<String, String> {
    let client = Client::new();
    
    // Calcular gap VPIP-PFR
    let vpip = data.vpip.parse::<f64>().unwrap_or(0.0);
    let pfr = data.pfr.parse::<f64>().unwrap_or(0.0);
    let gap = vpip - pfr;
    
    // Determinar etiqueta de gap
    let gap_label = if gap < 4.0 {
        "mínimo (estilo TAG)"
    } else if gap < 8.0 {
        "moderado"
    } else if gap < 12.0 {
        "notable (muchos calls)"
    } else {
        "extremo (muy pasivo)"
    };
    
    // Nombre del jugador para el informe
    let nombre_jugador = &data.player_name;
    
    // Crear prompt para GPT
    let prompt = create_analysis_prompt(nombre_jugador, gap_label, &data);
    
    // Verificar si hay una API key válida
    if api_key.trim().is_empty() {
        return Err("Se requiere una API key de OpenAI para realizar el análisis".to_string());
    }

    // Preparar la solicitud para la API de OpenAI
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());
    
    let openai_url = "https://api.openai.com/v1/chat/completions";
    
    // Crear la estructura de solicitud para la API de GPT
    let request_body = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 300,
        "temperature": 0.7
    });
    
    // Realizar la solicitud a la API de OpenAI
    for attempt in 0..3 {
        match client.post(openai_url)
            .headers(headers.clone())
            .json(&request_body)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        let response_body = response.json::<serde_json::Value>().await
                            .map_err(|e| format!("Error al decodificar respuesta: {}", e))?;
                        
                        // Extraer la respuesta generada
                        if let Some(choices) = response_body["choices"].as_array() {
                            if let Some(choice) = choices.get(0) {
                                if let Some(message) = choice["message"].as_object() {
                                    if let Some(content) = message["content"].as_str() {
                                        let full_response = content.trim();
                                        
                                        // Limpiar respuesta
                                        let analysis = if full_response.contains("📊 Stats") {
                                            full_response.split("📊 Stats").next().unwrap_or(full_response).trim()
                                        } else {
                                            full_response
                                        };
                                        
                                        return Ok(analysis.to_string());
                                    }
                                }
                            }
                        }
                        
                        return Err("No se pudo extraer la respuesta generada por GPT".to_string());
                    } else {
                        // Si no es el último intento, esperar y volver a intentar
                        if attempt < 2 {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            continue;
                        }
                        
                        return Err(format!("Error en la API de GPT: {}", response.status()));
                    }
                },
                Err(e) => {
                    // Si no es el último intento, esperar y volver a intentar
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        continue;
                    }
                    
                    return Err(format!("Error de conexión con GPT: {}", e));
                },
            }
    }
    
    Err("No se pudo obtener respuesta de la API de GPT después de varios intentos".to_string())
}

// Función para crear el prompt para análisis con GPT
fn create_analysis_prompt(nombre_jugador: &str, gap_label: &str, data: &PlayerStats) -> String {
    format!(
        r#"Eres un jugador profesional de cash online (NL50–NL100). Vas a analizar estadísticas de un oponente y generar un informe **corto, claro y accionable**, como si fuera una nota para otro reg en Discord.

🎯 Estilo directo, sin relleno, sin explicaciones teóricas. Evita tecnicismos largos. Usa lenguaje real de poker: "LAG", "se frena en turn", "flotar flop", "3B light", "spots CO vs BTN", etc.

📌 Evalúa stats **en conjunto**, no por separado. Ejemplos:
- VPIP alto + PFR bajo = pasivo.
- C-Bet flop alta + Turn baja = agresión inconsistente.
- WTSD alto + WSD bajo = paga mucho, gana poco.
- Fold al 3-Bet solo es leak si es >65% o <35%, o no cuadra con su estilo.

📌 Gap VPIP–PFR detectado: {gap_label}

📌 Si tiene menos de 1000 manos, di que el sample es bajo y que los reads son preliminares.

❌ No incluyas ninguna lista de estadísticas numéricas al final ni pongas "Stats clave". Solo el análisis.

---

📄 FORMATO EXACTO DEL INFORME:

🎯 Informe sobre {nombre_jugador}:

1️⃣ Estilo de juego: 
[Estilo en 1–2 líneas, con términos comunes entre regs]

2️⃣ Errores explotables:
- [Leak 1 corto]
- [Leak 2 corto]
- [Leak 3 corto]

3️⃣ Cómo explotarlo:
[Ajustes concisos, estilo "3Btea más en BTN", "flota flop seco", etc.]

---

📊 Stats disponibles:
- Manos: {total_manos}
- BB/100: {bb_100}
- Ganancias USD: {win_usd}
- VPIP: {vpip}%
- PFR: {pfr}%
- 3-Bet: {three_bet}%
- Fold to 3-Bet: {fold_to_3bet_pct}%
- 4-Bet: {four_bet_preflop_pct}%
- Fold to 4-Bet: {fold_to_4bet_pct}%
- C-Bet Flop: {cbet_flop}%
- C-Bet Turn: {cbet_turn}%
- WWSF: {wwsf}%
- WTSD: {wtsd}%
- WSD: {wsd}%
- Limp Preflop: {limp_pct}%
- Limp-Raise: {limp_raise_pct}%
- Fold to Flop C-Bet: {fold_to_flop_cbet_pct}%
- Fold to Turn C-Bet: {fold_to_turn_cbet_pct}%
- Probe Bet Turn: {probe_bet_turn_pct}%
- Fold to River Bet: {fold_to_river_bet_pct}%
- Bet River: {bet_river_pct}%
- Overbet Turn: {overbet_turn_pct}%
- Overbet River: {overbet_river_pct}%
- WSDwBR: {wsdwbr_pct}%"#,
        nombre_jugador = nombre_jugador,
        gap_label = gap_label,
        total_manos = data.total_manos,
        bb_100 = data.bb_100,
        win_usd = data.win_usd,
        vpip = data.vpip,
        pfr = data.pfr,
        three_bet = data.three_bet,
        fold_to_3bet_pct = data.fold_to_3bet_pct,
        four_bet_preflop_pct = data.four_bet_preflop_pct.as_ref().unwrap_or(&"0".to_string()),
        fold_to_4bet_pct = data.fold_to_4bet_pct.as_ref().unwrap_or(&"0".to_string()),
        cbet_flop = data.cbet_flop,
        cbet_turn = data.cbet_turn,
        wwsf = data.wwsf,
        wtsd = data.wtsd,
        wsd = data.wsd,
        limp_pct = data.limp_pct.as_ref().unwrap_or(&"0".to_string()),
        limp_raise_pct = data.limp_raise_pct.as_ref().unwrap_or(&"0".to_string()),
        fold_to_flop_cbet_pct = data.fold_to_flop_cbet_pct,
        fold_to_turn_cbet_pct = data.fold_to_turn_cbet_pct,
        probe_bet_turn_pct = data.probe_bet_turn_pct.as_ref().unwrap_or(&"0".to_string()),
        fold_to_river_bet_pct = data.fold_to_river_bet_pct.as_ref().unwrap_or(&"0".to_string()),
        bet_river_pct = data.bet_river_pct.as_ref().unwrap_or(&"0".to_string()),
        overbet_turn_pct = data.overbet_turn_pct.as_ref().unwrap_or(&"0".to_string()),
        overbet_river_pct = data.overbet_river_pct.as_ref().unwrap_or(&"0".to_string()),
        wsdwbr_pct = data.wsdwbr_pct.as_ref().unwrap_or(&"0".to_string()),
    )
}