import React from "react";
import { Box, Heading, Text, Icon } from "@chakra-ui/react";
import { FaCog } from "react-icons/fa";

export default function Configuracion() {
  return (
    <Box>
      <Heading size="lg" mb={4}>
        <Icon as={FaCog} mr={2} />
        Configuración
      </Heading>
      <Text fontSize="lg">
        Ajusta tus preferencias de PokerProTrack: tema, notificaciones y más.
      </Text>
      <Text mt={8} color="gray.500">
        (Próximamente: cambios de contraseña, tema oscuro, integración avanzada)
      </Text>
    </Box>
  );
}
