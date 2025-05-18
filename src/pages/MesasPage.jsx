// src/pages/MesasPage.jsx
import React from 'react';
import { Box, Container } from '@chakra-ui/react';
import MesasDetectadas from '../components/mesas/MesasDetectadas';
import ProfileDetector from '../components/mesas/ProfileDetector';

const MesasPage = () => {
  return (
    <Container maxW="container.xl" py={5}>
      <Box borderRadius="lg" bg="white" boxShadow="md" p={5}>
        <MesasDetectadas />
        <ProfileDetector />
      </Box>
    </Container>
  );
};

export default MesasPage;