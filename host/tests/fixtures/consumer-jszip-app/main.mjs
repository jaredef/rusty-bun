import JSZip from "jszip";
const z = new JSZip();
z.file("hello.txt", "world");
process.stdout.write(JSON.stringify({ hasFile: typeof z.file, hasGen: typeof z.generateAsync }) + "\n");
