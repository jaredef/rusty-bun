// svg-path-parser ^1 — parse SVG path "d" attribute into commands.
import { parseSVG } from "svg-path-parser";

const lines = [];
lines.push("1 " + JSON.stringify(parseSVG("M 0 0 L 10 10")));
lines.push("2 " + JSON.stringify(parseSVG("M0,0 H10 V10 Z")));
lines.push("3 " + JSON.stringify(parseSVG("M0 0 C 10 10 20 20 30 30")));
lines.push("4 " + JSON.stringify(parseSVG("m 5 5 l 10 0 l 0 10 z")));
lines.push("5 " + parseSVG("M 0 0 L 5 5").length);
lines.push("6 " + JSON.stringify(parseSVG("M 0 0 A 10 10 0 0 1 5 5")));

process.stdout.write(lines.join("\n") + "\n");
