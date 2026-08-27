import { describe, expect, it } from "vitest";

import { parseServerHealth } from "./index";

describe("server health contract", () => {
  it("accepts the healthy server response", () => {
    expect(
      parseServerHealth({
        service: "hemo-tracker-server",
        status: "ok",
      }),
    ).toEqual({
      service: "hemo-tracker-server",
      status: "ok",
    });
  });

  it("rejects an unknown status", () => {
    expect(() =>
      parseServerHealth({
        service: "hemo-tracker-server",
        status: "unhealthy",
      }),
    ).toThrow();
  });
});
