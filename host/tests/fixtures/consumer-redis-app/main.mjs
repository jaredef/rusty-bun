import { createClient } from "redis";
const client = createClient({ url: "redis://localhost:6379" });
process.stdout.write(JSON.stringify({
  hasCreateClient: typeof createClient,
  hasOn: typeof client.on,
  hasConnect: typeof client.connect,
  hasQuit: typeof client.quit,
  hasGet: typeof client.get,
  hasSet: typeof client.set,
}) + "\n");
