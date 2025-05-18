import React from "react";
import { useAuth } from "../context/AuthContext";
import { Box, Heading, Text, Button } from "@chakra-ui/react";

export default function Dashboard() {
  const { user, logout } = useAuth();

  return (
    <Box p={8}>
      <Heading>Dashboard PokerProTrack</Heading>
      <Text mt={4}>Bienvenido, {user?.nombre || user?.email}!</Text>
      <Text>Tu nivel: <b>{user?.suscripcion}</b></Text>
      <Button mt={6} colorScheme="red" onClick={logout}>
        Cerrar sesión
      </Button>
      {/* Aquí después agregas widgets, análisis, historial, etc */}
    </Box>
  );
}
