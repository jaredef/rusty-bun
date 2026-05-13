import { expand } from "dotenv-expand";
process.stdout.write(JSON.stringify({ type: typeof expand }) + "\n");
