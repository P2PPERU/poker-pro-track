import React from "react";
import { BrowserRouter, Routes, Route, Navigate, Link } from "react-router-dom";
import { ChakraProvider, Box, Flex, Button } from "@chakra-ui/react";
import { AuthProvider, useAuth } from "./context/AuthContext";
import LoginPage from "./pages/LoginPage";
import Dashboard from "./pages/Dashboard"; // Crea esta página
// Estas páginas puedes crearlas como placeholder por ahora
import Mesas from "./pages/MesasPage";         // Crea Mesas.jsx (placeholder)
import Historial from "./pages/Historial"; // Crea Historial.jsx (placeholder)
import Configuracion from "./pages/Configuracion"; // Crea Configuracion.jsx (placeholder)

function PrivateRoute({ children }) {
  const { token } = useAuth();
  return token ? children : <Navigate to="/login" />;
}

function Layout({ children }) {
  return (
    <Flex direction="row" h="100vh">
      <Box w="220px" bg="gray.900" color="white" p={6}>
        <Flex direction="column" gap={4}>
          <Link to="/dashboard"><Button w="full" colorScheme="blue" variant="solid">Dashboard</Button></Link>
          <Link to="/mesas"><Button w="full" colorScheme="blue" variant="ghost">Mesas</Button></Link>
          <Link to="/historial"><Button w="full" colorScheme="blue" variant="ghost">Historial</Button></Link>
          <Link to="/configuracion"><Button w="full" colorScheme="blue" variant="ghost">Configuración</Button></Link>
        </Flex>
      </Box>
      <Box flex="1" p={8} bg="gray.50" minH="100vh">{children}</Box>
    </Flex>
  );
}

export default function App() {
  return (
    <ChakraProvider>
      <AuthProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route
              path="/*"
              element={
                <PrivateRoute>
                  <Layout>
                    <Routes>
                      <Route path="/dashboard" element={<Dashboard />} />
                      <Route path="/mesas" element={<Mesas />} />
                      <Route path="/historial" element={<Historial />} />
                      <Route path="/configuracion" element={<Configuracion />} />
                      <Route path="*" element={<Navigate to="/dashboard" />} />
                    </Routes>
                  </Layout>
                </PrivateRoute>
              }
            />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </ChakraProvider>
  );
}
