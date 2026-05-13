// pino-pretty shape probe.
import pinoPretty from "pino-pretty";
process.stdout.write(JSON.stringify({
  hasPretty: typeof pinoPretty === "function",
}) + "\n");
