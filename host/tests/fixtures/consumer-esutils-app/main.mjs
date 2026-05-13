import esutils from "esutils";
process.stdout.write(JSON.stringify({ isIdent: esutils.keyword.isReservedWordES6("for", true) }) + "\n");
