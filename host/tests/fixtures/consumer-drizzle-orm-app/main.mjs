import { sql } from "drizzle-orm";

// Drizzle's basic shape: build a SQL query expression.
const q = sql`select ${42} as x`;
process.stdout.write(JSON.stringify({
  hasSql: typeof sql === "function",
  qIsObject: typeof q === "object" && q !== null,
  qHasQueryChunks: Array.isArray(q.queryChunks),
}) + "\n");
