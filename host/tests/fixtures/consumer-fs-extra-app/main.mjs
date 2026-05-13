import fse from "fs-extra";
import { join } from "node:path";
import { tmpdir } from "node:os";

const tmp = join(tmpdir(), "rusty-bun-fse-" + (process.pid || 1));

fse.removeSync(tmp);
fse.ensureDirSync(tmp);
fse.writeJsonSync(join(tmp, "data.json"), { a: 1, b: [1, 2, 3] });
const read = fse.readJsonSync(join(tmp, "data.json"));
const exists = fse.pathExistsSync(join(tmp, "data.json"));
const missing = !fse.pathExistsSync(join(tmp, "nope.txt"));
fse.removeSync(tmp);

process.stdout.write(JSON.stringify({ read, exists, missing }) + "\n");
