// src/services/auth.js
export async function login(email, password) {
  const response = await fetch("http://localhost:3000/api/auth/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email, password }),
  });

  if (!response.ok) {
    // Intenta extraer el mensaje de error del backend
    const error = await response.json().catch(() => ({}));
    throw new Error(error?.error || "Error de autenticación");
  }

  // Aquí te llega { mensaje, token, usuario }
  return await response.json();
}
