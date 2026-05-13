import { WebSocket as WS } from "ws";
process.stdout.write(JSON.stringify({ type: typeof WS, hasConnect: typeof WS === "function" }) + "\n");
