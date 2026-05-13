import upath from "upath";
process.stdout.write(JSON.stringify({ norm: upath.normalize("a\\b/c") }) + "\n");
