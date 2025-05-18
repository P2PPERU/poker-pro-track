// src/services/auth.js
import { loginUser } from './tauri';

export async function login(email, password) {
  try {
    // Llamada al comando Tauri para iniciar sesión
    const response = await loginUser(email, password);
    
    // Guardar datos del usuario en localStorage
    if (response && response.token && response.usuario) {
      localStorage.setItem("user", JSON.stringify(response.usuario));
      localStorage.setItem("token", response.token);
    }
    
    return response; // Devuelve { mensaje, token, usuario }
  } catch (error) {
    console.error("Error en inicio de sesión:", error);
    throw new Error(error || "Error de autenticación");
  }
}

export function logout() {
  localStorage.removeItem("user");
  localStorage.removeItem("token");
}

export function getToken() {
  return localStorage.getItem("token") || "";
}

export function getUser() {
  const user = localStorage.getItem("user");
  return user ? JSON.parse(user) : null;
}

export function isAuthenticated() {
  return !!getToken();
}