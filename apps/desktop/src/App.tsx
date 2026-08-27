import { Box, Heading, Stack, Text } from "@chakra-ui/react";

function App() {
  return (
    <Box as="main" minH="100vh" bg="bg" color="fg" px="8" py="12">
      <Stack gap="4" maxW="3xl">
        <Text
          color="teal.600"
          fontSize="sm"
          fontWeight="semibold"
          letterSpacing="wide"
          textTransform="uppercase"
        >
          Private by design
        </Text>
        <Heading as="h1" size="4xl">
          Hemo Tracker
        </Heading>
        <Text color="fg.muted" fontSize="xl">
          Local-first laboratory results with encrypted synchronization.
        </Text>
      </Stack>
    </Box>
  );
}

export default App;
