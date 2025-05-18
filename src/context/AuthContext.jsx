import React, { createContext, useContext, useState } from "react";
import { login as apiLogin } from "../services/auth";

const AuthContext = createContext();

export function AuthProvider({ children }) {
  const [user, setUser] = useState(() =>
    JSON.parse(localStorage.getItem("user")) || null
  );
  const [token, setToken] = useState(() => localStorage.getItem("token") || "");

  const login = async (email, password) => {
    const data = await apiLogin(email, password);
    setUser(data.usuario);
    setToken(data.token);

    localStorage.setItem("user", JSON.stringify(data.usuario));
    localStorage.setItem("token", data.token);
    return data.usuario;
  };

  const logout = () => {
    setUser(null);
    setToken("");
    localStorage.removeItem("user");
    localStorage.removeItem("token");
  };

  return (
    <AuthContext.Provider value={{ user, token, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
