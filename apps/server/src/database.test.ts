import { describe, expect, it } from "vitest";

import { sql } from "drizzle-orm";

import { openOperationsDatabase } from "./database";

describe("operations database", () => {
  it("opens an isolated SQLite database through Drizzle", async () => {
    const operations = openOperationsDatabase(":memory:");

    try {
      const result = await operations.database.get<{ answer: number }>(
        sql`select 42 as answer`,
      );

      expect(result).toEqual({ answer: 42 });
    } finally {
      operations.close();
    }
  });
});
