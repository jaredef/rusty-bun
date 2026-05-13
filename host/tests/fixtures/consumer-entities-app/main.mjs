import { decode, encode } from "entities";
process.stdout.write(JSON.stringify({ d: decode("&amp;"), e: encode("a&b") }) + "\n");
