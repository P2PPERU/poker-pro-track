import React, { useState } from "react";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";
import {
  Box, Button, FormControl, FormLabel, Input, VStack, Heading, Text, useToast
} from "@chakra-ui/react";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();
  const toast = useToast();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      await login(email, password);
      toast({ title: "¡Bienvenido!", status: "success", duration: 2000 });
      navigate("/dashboard");
    } catch (err) {
      toast({
        title: "Error",
        description: err.message,
        status: "error",
        duration: 3000,
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Box maxW="400px" mx="auto" mt="10vh" bg="white" boxShadow="md" p={8} borderRadius="xl">
      <Heading mb={6} size="lg">Iniciar sesión</Heading>
      <form onSubmit={handleSubmit}>
        <VStack spacing={4}>
          <FormControl isRequired>
            <FormLabel>Email</FormLabel>
            <Input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="tu@email.com"
            />
          </FormControl>
          <FormControl isRequired>
            <FormLabel>Contraseña</FormLabel>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
            />
          </FormControl>
          <Button
            type="submit"
            colorScheme="blue"
            isLoading={isLoading}
            loadingText="Entrando..."
            width="full"
          >
            Entrar
          </Button>
        </VStack>
      </form>
    </Box>
  );
}
