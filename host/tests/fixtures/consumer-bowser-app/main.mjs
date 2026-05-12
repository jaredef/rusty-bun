// bowser ^2 — User-Agent parser. Distinct axis: UA string → structured
// browser/OS/engine info.
import Bowser from "bowser";

const lines = [];

const uas = [
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
  "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
  "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
];

for (let i = 0; i < uas.length; i++) {
  const b = Bowser.parse(uas[i]);
  lines.push((i + 1) + " " + b.browser.name + "|" + b.os.name + "|" + b.platform.type + "|" + b.engine.name);
}

// 5: getBrowser shortcut
{
  const p = Bowser.getParser(uas[0]);
  lines.push("5 " + p.getBrowserName() + " " + (p.satisfies({ chrome: ">100" }) ? "ok" : "no"));
}

// 6: satisfies aggregate
{
  const p = Bowser.getParser(uas[2]);
  lines.push("6 ff=" + (p.satisfies({ firefox: ">=100" }) ? "yes" : "no"));
}

process.stdout.write(lines.join("\n") + "\n");
