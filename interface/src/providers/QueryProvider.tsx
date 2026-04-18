import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useMemo } from "react";

export function QueryProvider({ children }: { children: React.ReactNode }) {
  const query_client = useMemo(() => {
    return new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: 1000 * 10,
          retry: 3,
        },
        mutations: {
          retry: false,
        },
      },
    });
  }, []);

  return (
    <QueryClientProvider client={query_client}>{children}</QueryClientProvider>
  );
}
