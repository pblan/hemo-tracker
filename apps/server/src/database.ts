import { createClient } from "@libsql/client";
import { drizzle } from "drizzle-orm/libsql";

export function openOperationsDatabase(path: string) {
  const client = createClient({
    url: path === ":memory:" ? "file::memory:" : `file:${path}`,
  });

  return {
    close: () => client.close(),
    database: drizzle(client),
  };
}
