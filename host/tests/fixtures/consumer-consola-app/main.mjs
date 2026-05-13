import { consola } from "consola";
process.stdout.write(JSON.stringify({ hasConsola: typeof consola, hasInfo: typeof consola.info, hasError: typeof consola.error }) + "\n");
