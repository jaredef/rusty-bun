// Doc 716 §X empirical anchor: K1-IDENTITY closure of fs/promises.
// Each method wraps its wired sync counterpart; cut moved from
// "L5 throws" to "[WIRED-full]." Zero substrate change.
import { promises as fs } from "node:fs";
import path from "node:path";
import os from "node:os";

const tmp = path.join(os.tmpdir(), "rb-fsp-" + process.pid + "-" + Date.now());
await fs.mkdir(tmp, { recursive: true });

const f1 = path.join(tmp, "a.txt");
const f2 = path.join(tmp, "b.txt");
const f3 = path.join(tmp, "c.txt");

await fs.writeFile(f1, "hello");
const c1 = await fs.readFile(f1, "utf8");
await fs.appendFile(f1, " world");
const c2 = await fs.readFile(f1, "utf8");
await fs.copyFile(f1, f2);
const c3 = await fs.readFile(f2, "utf8");
await fs.rename(f2, f3);
const f2gone = await fs.access(f2).then(() => false).catch(() => true);
const f3exists = await fs.access(f3).then(() => true).catch(() => false);
const st = await fs.stat(f1);
const isF = typeof st.isFile === "function" ? st.isFile() : !!st.isFile;
await fs.chmod(f1, 0o644);
await fs.utimes(f1, new Date(), new Date());
await fs.unlink(f1);
await fs.unlink(f3);
await fs.rmdir(tmp);

process.stdout.write(JSON.stringify({ c1, c2, c3, f2gone, f3exists, isF }) + "\n");
