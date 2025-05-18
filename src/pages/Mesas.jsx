import React, { useState, useEffect } from "react";
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Badge,
  Button,
  Divider,
  useToast,
  IconButton,
} from "@chakra-ui/react";
import { RepeatIcon, SearchIcon } from "@chakra-ui/icons";

export default function Mesas() {
  // Estado local para las mesas activas (luego será data real de backend/OCR)
  const [mesas, setMesas] = useState([
    {
      id: 1,
      nombre: "Mesa X-Poker #1",
      estado: "En juego",
      jugadores: [
        { nick: "Player1", stack: 120, status: "activo" },
        { nick: "Player2", stack: 200, status: "activo" },
        { nick: "Player3", stack: 90, status: "ausente" },
      ],
      ultimaActualizacion: "Hace 5s",
    },
    {
      id: 2,
      nombre: "Mesa X-Poker #2",
      estado: "En espera",
      jugadores: [
        { nick: "PlayerA", stack: 160, status: "activo" },
        { nick: "PlayerB", stack: 80, status: "activo" },
      ],
      ultimaActualizacion: "Hace 10s",
    },
  ]);

  const [cargando, setCargando] = useState(false);
  const toast = useToast();

  // Simulación de actualización OCR
  const actualizarOCR = (mesaId) => {
    setCargando(true);
    // Aquí conectas el backend real (OCR/Python/Tauri)
    setTimeout(() => {
      setCargando(false);
      toast({
        title: "OCR actualizado",
        description: `La mesa ${mesaId} fue actualizada.`,
        status: "success",
        duration: 1800,
        isClosable: true,
      });
    }, 1500);
  };

  // Acción de análisis de mesa
  const analizarMesa = (mesaId) => {
    // Aquí conectarás con el análisis de jugadores/stats de esa mesa
    toast({
      title: "Análisis iniciado",
      description: `Se está analizando la mesa ${mesaId}`,
      status: "info",
      duration: 1800,
      isClosable: true,
    });
  };

  // Refrescar listado completo de mesas (simulado)
  const refrescarMesas = () => {
    setCargando(true);
    setTimeout(() => {
      setCargando(false);
      toast({
        title: "Listado actualizado",
        description: "Todas las mesas han sido refrescadas.",
        status: "success",
        duration: 1800,
        isClosable: true,
      });
      // Aquí iría fetch a backend para mesas reales
    }, 1200);
  };

  return (
    <Box maxW="900px" mx="auto" mt={8} p={6} bg="gray.50" borderRadius="2xl" boxShadow="xl">
      <HStack justify="space-between" mb={6}>
        <Heading size="lg" color="blue.700">Mesas Activas</Heading>
        <IconButton
          icon={<RepeatIcon />}
          onClick={refrescarMesas}
          colorScheme="blue"
          isLoading={cargando}
          aria-label="Actualizar listado"
        />
      </HStack>
      <Divider mb={4} />
      <VStack spacing={8} align="stretch">
        {mesas.length === 0 ? (
          <Box textAlign="center" p={10}>
            <Text color="gray.500" fontSize="xl">No hay mesas detectadas.</Text>
          </Box>
        ) : (
          mesas.map((mesa) => (
            <Box
              key={mesa.id}
              bg="white"
              p={5}
              borderRadius="xl"
              boxShadow="lg"
              _hover={{ boxShadow: "2xl", transform: "scale(1.01)", transition: ".2s" }}
            >
              <HStack justify="space-between">
                <VStack align="start" spacing={1}>
                  <Text fontWeight="bold" fontSize="lg">{mesa.nombre}</Text>
                  <HStack>
                    <Badge colorScheme={mesa.estado === "En juego" ? "green" : "gray"}>
                      {mesa.estado}
                    </Badge>
                    <Badge colorScheme="purple">{mesa.ultimaActualizacion}</Badge>
                  </HStack>
                </VStack>
                <HStack spacing={2}>
                  <Button
                    leftIcon={<RepeatIcon />}
                    size="sm"
                    colorScheme="teal"
                    variant="outline"
                    isLoading={cargando}
                    onClick={() => actualizarOCR(mesa.id)}
                  >
                    Actualizar OCR
                  </Button>
                  <Button
                    leftIcon={<SearchIcon />}
                    size="sm"
                    colorScheme="blue"
                    onClick={() => analizarMesa(mesa.id)}
                  >
                    Analizar Mesa
                  </Button>
                </HStack>
              </HStack>
              <Box mt={3} pl={1}>
                <Text fontSize="sm" fontWeight="bold" mb={2} color="gray.700">Jugadores:</Text>
                <HStack spacing={3} wrap="wrap">
                  {mesa.jugadores.map((jugador, i) => (
                    <Badge
                      key={jugador.nick + i}
                      colorScheme={jugador.status === "activo" ? "blue" : "gray"}
                      variant={jugador.status === "activo" ? "solid" : "subtle"}
                      fontSize="md"
                      px={3}
                      py={1}
                      borderRadius="md"
                    >
                      {jugador.nick} ({jugador.stack})
                    </Badge>
                  ))}
                </HStack>
              </Box>
            </Box>
          ))
        )}
      </VStack>
    </Box>
  );
}
