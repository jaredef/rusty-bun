import superagent from "superagent";
process.stdout.write(JSON.stringify({ hasGet: typeof superagent.get, hasPost: typeof superagent.post }) + "\n");
