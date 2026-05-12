// validator ^13 — string validators (email/url/uuid/iban/etc) + sanitizers.
// Distinct axis: a large suite of single-string predicates.
import validator from "validator";

const lines = [];

lines.push("1 email=" + validator.isEmail("foo@bar.com") + " bad=" + validator.isEmail("nope"));
lines.push("2 url=" + validator.isURL("https://example.com/path?q=1"));
lines.push("3 uuid=" + validator.isUUID("550e8400-e29b-41d4-a716-446655440000"));
lines.push("4 ip4=" + validator.isIP("127.0.0.1", 4) + " ip6=" + validator.isIP("::1", 6));
lines.push("5 alphanum=" + validator.isAlphanumeric("abc123") + " not=" + validator.isAlphanumeric("a b"));
lines.push("6 esc=" + validator.escape("<b>x&y</b>"));

process.stdout.write(lines.join("\n") + "\n");
