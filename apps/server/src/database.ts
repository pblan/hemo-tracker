import { createClient } from "@libsql/client";
import { drizzle } from "drizzle-orm/libsql";

export function openOperationsDatabase(url: string) {
  const client = createClient({ url });

  return {
    close: () => client.close(),
    database: drizzle(client),
  };
}
