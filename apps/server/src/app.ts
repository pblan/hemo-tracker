import type { ServerHealth } from "@hemo-tracker/contracts";

const healthyResponse: ServerHealth = {
  service: "hemo-tracker-server",
  status: "ok",
};

export function handleRequest(request: Request): Response {
  const { pathname } = new URL(request.url);

  if (request.method === "GET" && pathname === "/health") {
    return Response.json(healthyResponse);
  }

  return Response.json({ error: "Not found" }, { status: 404 });
}
