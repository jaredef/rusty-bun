import OpenAI from "openai";
const client = new OpenAI({ apiKey: "sk-test" });
process.stdout.write(JSON.stringify({ hasChat: typeof client.chat, hasCompletions: typeof client.completions }) + "\n");
