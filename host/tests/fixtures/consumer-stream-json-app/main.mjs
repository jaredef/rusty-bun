// stream-json shape probe.
import streamJson from "stream-json";
process.stdout.write(JSON.stringify({
  hasParser: typeof streamJson.parser === "function" || typeof streamJson.parser === "object",
  hasMake: typeof streamJson.make === "function" || typeof streamJson.default === "function" || typeof streamJson.default === "object",
}) + "\n");
