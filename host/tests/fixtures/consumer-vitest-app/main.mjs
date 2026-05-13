import * as vitest from "vitest";

process.stdout.write(JSON.stringify({
  hasDescribe: typeof vitest.describe === "function",
  hasIt: typeof vitest.it === "function",
  hasExpect: typeof vitest.expect === "function",
  hasTest: typeof vitest.test === "function",
  hasBeforeEach: typeof vitest.beforeEach === "function",
}) + "\n");
