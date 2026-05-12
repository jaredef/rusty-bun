// color ^5 — color conversion + manipulation (RGB/HSL/HSV/etc).
import Color from "color";

const lines = [];

lines.push("1 " + Color("#ff0000").hex());
lines.push("2 " + Color("red").rgb().string());
lines.push("3 " + Color("#00ff00").hsl().string(0));
lines.push("4 " + Color({ r: 255, g: 128, b: 64 }).hex());
lines.push("5 " + Color("#888888").lighten(0.5).hex());
lines.push("6 " + Color("#ff0000").mix(Color("#0000ff"), 0.5).hex());

process.stdout.write(lines.join("\n") + "\n");
