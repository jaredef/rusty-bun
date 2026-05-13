import BigNumber from "bignumber.js";
const r = new BigNumber("1").div("3").toFixed(10);
process.stdout.write(JSON.stringify({ r }) + "\n");
