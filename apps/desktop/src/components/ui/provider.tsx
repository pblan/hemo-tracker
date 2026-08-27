import { ChakraProvider, defaultSystem } from "@chakra-ui/react";
import { QueryClientProvider } from "@tanstack/react-query";
import type { PropsWithChildren } from "react";

import { queryClient } from "../../query-client";

export function Provider({ children }: PropsWithChildren) {
  return (
    <QueryClientProvider client={queryClient}>
      <ChakraProvider value={defaultSystem}>{children}</ChakraProvider>
    </QueryClientProvider>
  );
}
