use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;

use crate::window_manager::is_poker_table;

// Estructura para mantener el estado del detector
pub struct RightClickDetector {
    active: Arc<Mutex<bool>>,
    running: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl RightClickDetector {
    pub fn new() -> Self {
        RightClickDetector {
            active: Arc::new(Mutex::new(false)),
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
        }
    }

    pub fn start<R: Runtime>(&mut self, app_handle: AppHandle<R>) -> Result<(), String> {
        // Evitar iniciar múltiples hilos
        if let Ok(mut running) = self.running.lock() {
            if *running {
                return Ok(());
            }
            *running = true;
        } else {
            return Err("No se pudo bloquear el estado running".to_string());
        }

        // Clonar referencias para el hilo
        let active = Arc::clone(&self.active);
        let running = Arc::clone(&self.running);

        // Iniciar hilo para monitoreo
        self.thread_handle = Some(thread::spawn(move || {
            Self::run_detection_loop(app_handle, active, running);
        }));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // Marcar como inactivo para que el hilo se detenga
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        } else {
            return Err("No se pudo bloquear el estado running".to_string());
        }

        // Esperar a que el hilo termine
        if let Some(handle) = self.thread_handle.take() {
            match handle.join() {
                Ok(_) => Ok(()),
                Err(_) => Err("Error al esperar a que el hilo termine".to_string()),
            }
        } else {
            Ok(())
        }
    }

    pub fn set_active(&self, active: bool) -> Result<(), String> {
        if let Ok(mut state) = self.active.lock() {
            *state = active;
            Ok(())
        } else {
            Err("No se pudo bloquear el estado active".to_string())
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.lock().map(|guard| *guard).unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    fn run_detection_loop<R: Runtime>(
        app_handle: AppHandle<R>,
        active: Arc<Mutex<bool>>,
        running: Arc<Mutex<bool>>
    ) {
        // Ventana previamente bajo el cursor
        let mut last_hwnd: HWND = 0;
        let mut last_check_time = std::time::Instant::now();
        
        // Estado de clic derecho
        let mut right_button_down = false;

        // Bucle principal
        while running.lock().map(|guard| *guard).unwrap_or(false) {
            // Verificar si el detector está activo
            let is_active = active.lock().map(|guard| *guard).unwrap_or(false);
            
            if is_active {
                unsafe {
                    // Comprobar si el botón derecho está siendo presionado
                    let right_btn_state = GetAsyncKeyState(VK_RBUTTON as i32) as u16;
                    let right_btn_pressed = (right_btn_state & 0x8000) != 0;
                    
                    // Detectar cuando se presiona (transición de no presionado a presionado)
                    if right_btn_pressed && !right_button_down {
                        // Se ha detectado un clic derecho, obtener ventana bajo el cursor
                        let mut point: POINT = std::mem::zeroed();
                        if GetCursorPos(&mut point) != 0 {
                            let hwnd = WindowFromPoint(point);
                            
                            // Si no es la misma ventana que antes, verificar si es de póker
                            if hwnd != 0 && hwnd != last_hwnd {
                                // Obtener título de la ventana
                                let mut title: [u16; 512] = [0; 512];
                                let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
                                
                                if len > 0 {
                                    let title_str = String::from_utf16_lossy(&title[0..len as usize]);
                                    
                                    // Verificar si es un título de ventana de póker
                                    if is_poker_table(&title_str) {
                                        // Verificar si parece un perfil de jugador (clase de ventana)
                                        let mut class_name: [u16; 256] = [0; 256];
                                        let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
                                        
                                        if class_len > 0 {
                                            // Convertir el nombre de la clase a string
                                            let class_str = String::from_utf16_lossy(&class_name[0..class_len as usize]);
                                            
                                            // Calcular coordenadas relativas al cliente
                                            let mut client_point = point;
                                            ScreenToClient(hwnd, &mut client_point);
                                            
                                            // Emitir evento a la interfaz
                                            let _ = app_handle.emit_all("profile_right_click", serde_json::json!({
                                                "hwnd": hwnd,
                                                "title": title_str,
                                                "class": class_str,
                                                "x": client_point.x,
                                                "y": client_point.y,
                                                "screen_x": point.x,
                                                "screen_y": point.y
                                            }));
                                            
                                            // Actualizar ventana actual
                                            last_hwnd = hwnd;
                                            last_check_time = std::time::Instant::now();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Actualizar estado del botón
                    right_button_down = right_btn_pressed;
                }
            }
            
            // Si han pasado más de 5 segundos desde la última detección, resetear la ventana actual
            if last_check_time.elapsed().as_secs() > 5 {
                last_hwnd = 0;
            }
            
            // Dormir para no consumir demasiada CPU
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn run_detection_loop<R: Runtime>(
        _app_handle: AppHandle<R>,
        _active: Arc<Mutex<bool>>,
        running: Arc<Mutex<bool>>
    ) {
        // Para plataformas no-Windows, implementar una versión simulada o usar otra API
        while running.lock().map(|guard| *guard).unwrap_or(false) {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

// Implementar Drop para asegurar que el hilo se detenga cuando se destruya
impl Drop for RightClickDetector {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}