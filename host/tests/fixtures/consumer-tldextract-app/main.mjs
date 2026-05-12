// tldts ^7 — public suffix list-based domain parser.
import { parse, getDomain, getPublicSuffix } from "tldts";

const lines = [];

const r1 = parse("https://www.example.co.uk/path?q=1");
lines.push("1 d=" + r1.domain + " s=" + r1.subdomain + " ps=" + r1.publicSuffix);

const r2 = parse("foo.bar.example.com");
lines.push("2 d=" + r2.domain + " s=" + r2.subdomain);

lines.push("3 " + getDomain("a.b.c.example.org"));
lines.push("4 " + getPublicSuffix("www.amazon.co.jp"));

const r5 = parse("192.168.1.1");
lines.push("5 isIp=" + r5.isIp);

const r6 = parse("https://localhost:8080");
lines.push("6 d=" + r6.domain + " host=" + r6.hostname);

process.stdout.write(lines.join("\n") + "\n");
