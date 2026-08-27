import { handleRequest } from "./app";

const port = Number.parseInt(process.env.PORT ?? "3000", 10);

const server = Bun.serve({
  fetch: handleRequest,
  port,
});

console.info(`Hemo Tracker server listens on ${server.url.origin}`);
