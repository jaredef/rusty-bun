import log from "loglevel";
process.stdout.write(JSON.stringify({ hasInfo: typeof log.info, hasSetLevel: typeof log.setLevel }) + "\n");
