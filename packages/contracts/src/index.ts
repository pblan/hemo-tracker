import { z } from "zod";

const serverHealthSchema = z.object({
  service: z.literal("hemo-tracker-server"),
  status: z.literal("ok"),
});

export type ServerHealth = z.infer<typeof serverHealthSchema>;

export function parseServerHealth(value: unknown): ServerHealth {
  return serverHealthSchema.parse(value);
}
