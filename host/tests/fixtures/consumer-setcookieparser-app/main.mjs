// set-cookie-parser ^3 — parse Set-Cookie response headers.
import setCookie from "set-cookie-parser";

const lines = [];
const raw = [
  "sid=abc; Path=/; HttpOnly",
  "foo=bar; Max-Age=3600; Secure; SameSite=Strict",
];
lines.push("1 " + JSON.stringify(setCookie.parse(raw)));
lines.push("2 " + JSON.stringify(setCookie.parse("a=1; Domain=example.com; Expires=Wed, 21 Oct 2015 07:28:00 GMT")));
const combined = "x=1, y=2";
const split = setCookie.splitCookiesString(combined);
lines.push("3 " + JSON.stringify(split));
lines.push("4 " + JSON.stringify(setCookie.parseString("foo=bar; HttpOnly")));
const o = setCookie.parse(["a=1", "b=2"], { map: true });
lines.push("5 " + JSON.stringify(Object.keys(o).sort()));
lines.push("6 " + JSON.stringify(setCookie.parse("a=1; SameSite=Lax")));

process.stdout.write(lines.join("\n") + "\n");
