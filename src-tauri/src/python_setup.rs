use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use crate::error::AppError;

// Asegura que el entorno Python esté correctamente configurado
pub fn ensure_python_env() -> Result<(), AppError> {
    // Verificar que Python está instalado
    if !is_python_installed() {
        return Err(AppError::Config(
            "Python no está instalado o accesible en el PATH".to_string(),
        ));
    }

    // Obtener el directorio correcto de Python
    let python_dir = get_python_directory();
    println!("Usando directorio Python: {:?}", python_dir);
    
    // Crear directorios si no existen
    create_python_directories(&python_dir)?;

    // Verificar o crear archivo de requisitos
    create_requirements_file(&python_dir)?;

    // Verificar que el script OCR existe
    let ocr_script_path = python_dir.join("src").join("core").join("ocr_engine.py");
    println!("Buscando script OCR en: {:?}", ocr_script_path);
    
    if !ocr_script_path.exists() {
        return Err(AppError::Config(format!(
            "No se encontró el script OCR en: {}",
            ocr_script_path.display()
        )));
    } else {
        println!("Script OCR encontrado correctamente");
    }
    
    // Instalar dependencias
    install_python_dependencies(&python_dir)?;

    // Verificar que OCR está disponible
    check_ocr_installation()?;

    Ok(())
}

// Verifica si Python está instalado
fn is_python_installed() -> bool {
    // Intentar ejecutar Python
    let output = Command::new("python")
        .arg("--version")
        .output();

    // Si no funciona, intentar con python3
    if output.is_err() {
        let output = Command::new("python3")
            .arg("--version")
            .output();
            
        return output.is_ok() && output.unwrap().status.success();
    }

    output.is_ok() && output.unwrap().status.success()
}

// Obtiene el directorio de Python correcto
pub fn get_python_directory() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    // La carpeta python está en el directorio raíz del proyecto, un nivel por encima de src-tauri
    let parent_dir = current_dir.parent().unwrap_or(&current_dir);
    let python_dir = parent_dir.join("python");
    
    if python_dir.exists() {
        println!("Encontrada estructura en la raíz: {:?}", python_dir);
        return python_dir;
    }
    
    // Comprobar también en el directorio actual (por si acaso)
    let local_python = current_dir.join("python");
    if local_python.exists() {
        println!("Encontrada estructura en el directorio actual: {:?}", local_python);
        return local_python;
    }
    
    // Si no existe, usar la ruta en la raíz
    println!("No se encontró estructura existente, usando: {:?}", python_dir);
    python_dir
}

// Crea los directorios necesarios para Python
fn create_python_directories(python_dir: &PathBuf) -> Result<(), AppError> {
    // Crear directorio principal si no existe
    if !python_dir.exists() {
        println!("Creando directorio principal: {:?}", python_dir);
        fs::create_dir_all(&python_dir)
            .map_err(|e| AppError::Io(e))?;
    }
    
    // Crear estructura de directorios para módulos Python
    let module_dir = python_dir.join("src").join("core");
    if !module_dir.exists() {
        println!("Creando directorio de módulos: {:?}", module_dir);
        fs::create_dir_all(&module_dir)
            .map_err(|e| AppError::Io(e))?;
    }
    
    // Crear carpeta para capturas
    let captures_dir = python_dir.join("capturas");
    if !captures_dir.exists() {
        println!("Creando directorio de capturas: {:?}", captures_dir);
        fs::create_dir_all(&captures_dir)
            .map_err(|e| AppError::Io(e))?;
    }
    
    // Crear carpeta para logs
    let logs_dir = python_dir.join("logs");
    if !logs_dir.exists() {
        println!("Creando directorio de logs: {:?}", logs_dir);
        fs::create_dir_all(&logs_dir)
            .map_err(|e| AppError::Io(e))?;
    }
    
    // Crear archivo __init__.py en cada directorio
    let src_dir = python_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)
            .map_err(|e| AppError::Io(e))?;
    }
    
    let init_files = [
        src_dir.join("__init__.py"),
        module_dir.join("__init__.py"),
    ];
    
    for init_file in init_files.iter() {
        if !init_file.exists() {
            fs::write(init_file, "# Módulo inicializador\n")
                .map_err(|e| AppError::Io(e))?;
        }
    }
    
    Ok(())
}

// Crea o verifica el archivo de requisitos
fn create_requirements_file(python_dir: &PathBuf) -> Result<(), AppError> {
    let requirements_path = python_dir.join("requirements.txt");
    
    // Si el archivo no existe, crear con dependencias básicas
    if !requirements_path.exists() {
        println!("Creando archivo requirements.txt: {:?}", requirements_path);
        let requirements_content = r#"# Dependencias para PokerProTrack OCR
paddleocr~=2.6.0
pillow~=9.4.0
numpy~=1.24.2
pywin32~=306; platform_system=="Windows"
opencv-python-headless~=4.7.0.72
"#;
        
        fs::write(&requirements_path, requirements_content)
            .map_err(|e| AppError::Io(e))?;
    }
    
    Ok(())
}

// Instala las dependencias de Python
fn install_python_dependencies(python_dir: &PathBuf) -> Result<(), AppError> {
    let requirements_path = python_dir.join("requirements.txt");
    
    // Verificar que el archivo de requisitos existe
    if !requirements_path.exists() {
        return Err(AppError::Config(format!(
            "No se encontró el archivo de requisitos en {}",
            requirements_path.display()
        )));
    }
    
    // Instalar dependencias con pip
    println!("Instalando dependencias Python desde: {:?}", requirements_path);
    let output = Command::new("pip")
        .arg("install")
        .arg("-r")
        .arg(&requirements_path)
        .output()
        .map_err(|e| AppError::Io(e))?;
        
    if !output.status.success() {
        println!("Advertencia: Problemas al instalar dependencias Python");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

// Verifica que OCR está correctamente instalado
fn check_ocr_installation() -> Result<(), AppError> {
    println!("Verificando instalación de PaddleOCR...");
    
    // Intentar importar PaddleOCR
    let output = Command::new("python")
        .arg("-c")
        .arg("try: import paddleocr; print('OK'); except Exception as e: print(f'Error: {e}')")
        .output()
        .map_err(|e| AppError::Io(e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if !output.status.success() || !stdout.starts_with("OK") {
        println!("Advertencia: PaddleOCR no está correctamente instalado");
        println!("Salida: {}", stdout);
        
        // Intentar una instalación directa como último recurso
        let _ = Command::new("pip")
            .arg("install")
            .arg("paddleocr")
            .output();
    }
    
    Ok(())
}