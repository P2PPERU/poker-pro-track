import React from "react";
import { Box, Heading, Text, Icon } from "@chakra-ui/react";
import { FaHistory } from "react-icons/fa";

export default function Historial() {
  return (
    <Box>
      <Heading size="lg" mb={4}>
        <Icon as={FaHistory} mr={2} />
        Historial de Análisis
      </Heading>
      <Text fontSize="lg">
        Aquí podrás ver el historial de manos, partidas y análisis realizados con PokerProTrack.
      </Text>
      <Text mt={8} color="gray.500">
        (Esta sección mostrará el listado y permitirá exportar o revisar detalles)
      </Text>
    </Box>
  );
}
