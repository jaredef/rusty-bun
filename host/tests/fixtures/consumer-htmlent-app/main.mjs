import { encode, decode } from "html-entities";

const lines = [];

lines.push("1 enc=" + encode("<script>alert('xss')</script>"));
lines.push("2 dec=" + decode("&lt;p&gt;hello&amp;world&lt;/p&gt;"));
lines.push("3 named=" + encode("foo&bar<", { level: "html5", mode: "nonAsciiPrintable" }));
lines.push("4 numeric=" + decode("&#65;&#66;&#67; &#x41;&#x42;&#x43;"));
lines.push("5 round=" + (decode(encode("<a>&'\"")) === "<a>&'\""));
lines.push("6 nbsp=" + decode("&copy; &amp; &nbsp;").length);

process.stdout.write(lines.join("\n") + "\n");
