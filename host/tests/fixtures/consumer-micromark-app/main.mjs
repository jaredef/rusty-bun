import { micromark } from "micromark";
const r = micromark("# Hello\nworld");
process.stdout.write(JSON.stringify({ r: r.trim() }) + "\n");
