import { describe, expect, it } from "vitest";

import { parseServerHealth } from "@hemo-tracker/contracts";

import { handleRequest } from "./app";

describe("server request handler", () => {
  it("returns a contract-valid health response", async () => {
    const response = handleRequest(new Request("http://localhost/health"));

    expect(response.status).toBe(200);
    await expect(response.json().then(parseServerHealth)).resolves.toEqual({
      service: "hemo-tracker-server",
      status: "ok",
    });
  });

  it("returns not found for an unknown route", () => {
    const response = handleRequest(new Request("http://localhost/missing"));

    expect(response.status).toBe(404);
  });
});
