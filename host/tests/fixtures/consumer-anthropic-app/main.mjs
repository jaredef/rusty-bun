import Anthropic from "@anthropic-ai/sdk";
const client = new Anthropic({ apiKey: "sk-test" });
process.stdout.write(JSON.stringify({ hasMessages: typeof client.messages }) + "\n");
