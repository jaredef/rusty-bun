import through2 from "through2";
const t = through2.obj((chunk, enc, cb) => cb(null, chunk));
process.stdout.write(JSON.stringify({ hasWrite: typeof t.write, hasRead: typeof t.read }) + "\n");
