// bignumber.js ^11 — arbitrary-precision decimal arithmetic (distinct
// from decimal.js already tested; different API surface).
import BigNumber from "bignumber.js";

const lines = [];
lines.push("1 " + new BigNumber("0.1").plus("0.2").toString());
lines.push("2 " + new BigNumber("12345678901234567890").times("2").toFixed());
lines.push("3 " + new BigNumber(10).pow(50).toString());
lines.push("4 " + new BigNumber(1).div(3).toFixed(20));
lines.push("5 " + new BigNumber("0.999999999999999").isEqualTo("0.999999999999999"));
lines.push("6 " + new BigNumber("ff", 16).toString());

process.stdout.write(lines.join("\n") + "\n");
