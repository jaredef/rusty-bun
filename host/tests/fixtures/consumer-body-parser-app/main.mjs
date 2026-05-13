// body-parser shape probe — full express integration would exercise
// the cooperative-loop reactor edge.
import bodyParser from "body-parser";
process.stdout.write(JSON.stringify({
  hasJson: typeof bodyParser.json === "function",
  hasUrlencoded: typeof bodyParser.urlencoded === "function",
  hasRaw: typeof bodyParser.raw === "function",
  hasText: typeof bodyParser.text === "function",
}) + "\n");
