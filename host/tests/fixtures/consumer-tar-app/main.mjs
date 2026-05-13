import * as tar from "tar";
process.stdout.write(JSON.stringify({ hasC: typeof tar.c, hasX: typeof tar.x, hasT: typeof tar.t }) + "\n");
