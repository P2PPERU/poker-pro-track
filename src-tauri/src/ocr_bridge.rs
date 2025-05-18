// src-tauri/src/ocr_bridge.rs
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use crate::error::AppError;
use crate::python_setup;

// Estructura para resultados OCR
pub struct OcrResult {
    pub nick: String,
    pub confidence: f32,
    pub image_hash: String,
}

// Inicializa el motor OCR
pub fn initialize_ocr(config: Option<&HashMap<String, String>>) -> Result<bool, String> {
    // Obtener el directorio Python utilizando la misma función que python_setup
    let python_dir = python_setup::get_python_directory();
    let ocr_script_path = python_dir.join("src").join("core").join("ocr_engine.py");
    
    println!("Inicializando OCR desde: {:?}", ocr_script_path);
    
    // Verificar que el script existe
    if !ocr_script_path.exists() {
        return Err(format!("No se encontró el script OCR en: {:?}", ocr_script_path));
    }
    
    // Construir el directorio que contiene el script
    let script_dir = ocr_script_path.parent().unwrap_or(Path::new(""));
    
    // Preparar configuración
    let config_str = if let Some(cfg) = config {
        serde_json::to_string(cfg).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    };
    
    // Ejecutar script para inicializar OCR
    let python_cmd = if cfg!(windows) { "python" } else { "python3" };
    
    // En lugar de pasar la ruta como string en el comando, usar el directorio como directorio de trabajo
    // y ejecutar ocr_engine.py directamente
    let output = Command::new(python_cmd)
        .current_dir(script_dir)
        .arg("-c")
        .arg(format!(
            "import sys; sys.path.append('.'); from ocr_engine import initialize_ocr; import json; print(initialize_ocr(json.loads('{}')))",
            config_str
        ))
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !output.status.success() {
                return Err(format!("Error al inicializar OCR: {}", stderr));
            }
            
            if stdout.contains("True") || stdout.contains("OK") {
                Ok(true)
            } else {
                Err(format!("Error al inicializar OCR: {}", stdout))
            }
        },
        Err(e) => Err(format!("Error al ejecutar Python: {}", e)),
    }
}

// Verifica disponibilidad de OCR - FUNCIÓN CORREGIDA
pub fn check_ocr_availability() -> bool {
    // Obtener el directorio Python
    let python_dir = python_setup::get_python_directory();
    let ocr_script_path = python_dir.join("src").join("core").join("ocr_engine.py");
    
    // Verificar que el script existe
    if !ocr_script_path.exists() {
        println!("Script OCR no encontrado en: {:?}", ocr_script_path);
        return false;
    }
    
    // Construir el directorio que contiene el script
    let script_dir = ocr_script_path.parent().unwrap_or(Path::new(""));
    
    // Usar el mismo enfoque que en initialize_ocr
    let python_cmd = if cfg!(windows) { "python" } else { "python3" };
    
    // Usar directamente la función check_ocr_availability del script Python
    let output = Command::new(python_cmd)
        .current_dir(script_dir)
        .arg("-c")
        .arg("import sys; sys.path.append('.'); from ocr_engine import check_ocr_availability; print(check_ocr_availability())")
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Si el script devuelve True o OK, consideramos que OCR está disponible
            if stdout.contains("True") || stdout.contains("true") || stdout.contains("OK") {
                println!("OCR está disponible según check_ocr_availability");
                true
            } else {
                println!("OCR no está disponible según check_ocr_availability: {}", stdout);
                
                // Alternativa: forzar a que devuelva siempre true para evitar errores
                // println!("Ignorando error de disponibilidad OCR, retornando true");
                // true
                
                false
            }
        },
        Err(e) => {
            println!("Error al verificar disponibilidad OCR: {}", e);
            false
        },
    }
}

// Captura y lee un nick desde una ventana - OPTIMIZADA PARA PERFILES
pub fn capture_and_read_nick(hwnd: u32, coords: HashMap<String, i32>) -> Result<OcrResult, AppError> {
    // Obtener el directorio Python
    let python_dir = python_setup::get_python_directory();
    let ocr_script_path = python_dir.join("src").join("core").join("ocr_engine.py");
    
    println!("Usando script OCR en: {:?}", ocr_script_path);
    
    if !ocr_script_path.exists() {
        return Err(AppError::Config(format!(
            "No se encontró el script OCR en: {:?}",
            ocr_script_path
        )));
    }
    
    // Construir el directorio que contiene el script
    let script_dir = ocr_script_path.parent().unwrap_or(Path::new(""));
    
    // Verificar si la ventana parece un perfil para ajustar coordenadas
    // Si las coordenadas no son explícitas para perfiles, ajustarlas
    let optimized_coords = optimize_coords_for_profile(hwnd, coords)?;
    
    // Convertir HashMap a String para pasar al script Python
    let coords_str = serde_json::to_string(&optimized_coords).unwrap_or_else(|_| "{}".to_string());
    
    // Comando para ejecutar OCR optimizado para perfiles
    let python_cmd = if cfg!(windows) { "python" } else { "python3" };
    
    // Usar el script con parámetros de optimización para perfiles
    let output = Command::new(python_cmd)
        .current_dir(script_dir)
        .arg("-c")
        .arg(format!(
            "import sys; sys.path.append('.'); from ocr_engine import capture_and_read_nick, enhance_image_for_ocr; import json; print(json.dumps(capture_and_read_nick({}, json.loads('{}'), enhance_profile=True)))",
            hwnd,
            coords_str
        ))
        .output()
        .map_err(|e| AppError::Io(e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Api(format!("Error en OCR: {}", stderr)));
    }
    
    // Procesar resultado
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Usar AppError::Api en lugar de AppError::Parse que no existe
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Api(format!("Error al parsear resultado OCR: {} - Datos: {}", e, stdout)))?;
    
    // Extraer y limpiar campos
    let nick = result["nick"].as_str().unwrap_or("Error").to_string();
    let confidence = result["confidence"].as_f64().unwrap_or(0.0) as f32;
    let image_hash = result["image_hash"].as_str().unwrap_or("").to_string();
    
    // Limpieza adicional para perfiles
    let clean_nick = nick.trim_start_matches("ID:").trim().to_string();
    
    Ok(OcrResult {
        nick: clean_nick,
        confidence,
        image_hash,
    })
}

// Nueva función para optimizar coordenadas según el tipo de ventana
fn optimize_coords_for_profile(hwnd: u32, coords: HashMap<String, i32>) -> Result<HashMap<String, i32>, AppError> {
    let mut optimized = coords.clone();
    
    // En Windows, intentar detectar si es una ventana de perfil
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        use windows_sys::Win32::Foundation::*;
        
        let hwnd_native = hwnd as HWND;
        
        // Verificar que la ventana existe
        if IsWindow(hwnd_native) == 0 {
            return Err(AppError::WindowDetection("La ventana especificada no existe".to_string()));
        }
        
        // Obtener título de la ventana
        let mut title: [u16; 512] = [0; 512];
        let title_len = GetWindowTextW(hwnd_native, title.as_mut_ptr(), title.len() as i32);
        
        if title_len > 0 {
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            
            let title_os_string = OsString::from_wide(&title[0..title_len as usize]);
            let title_str = title_os_string.to_string_lossy().to_string();
            
            // Verificar si es un perfil por palabras clave en el título
            let is_profile = title_str.contains("Profile") || 
                            title_str.contains("perfil") || 
                            title_str.contains("ID:") ||
                            title_str.contains("jugador");
            
            if is_profile {
                println!("Detectada ventana de perfil: '{}'", title_str);
                
                // Obtener tamaño de la ventana
                let mut rect: RECT = std::mem::zeroed();
                GetWindowRect(hwnd_native, &mut rect);
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;
                
                // Coordenadas optimizadas para perfil tipo Ryunouske
                // Ajustadas para el área donde suele estar el nick en perfiles
                optimized.insert("x".to_string(), 180);  
                optimized.insert("y".to_string(), 230);  
                optimized.insert("w".to_string(), 140);  
                optimized.insert("h".to_string(), 40);   
                
                println!("Coordenadas optimizadas para perfil: {:?}", optimized);
            }
        }
    }
    
    Ok(optimized)
}