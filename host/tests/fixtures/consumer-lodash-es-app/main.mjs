import { chunk, uniq, sortBy } from "lodash-es";
process.stdout.write(JSON.stringify({ chunk:chunk([1,2,3,4],2), uniq:uniq([1,1,2,2,3]), sorted:sortBy([{n:2},{n:1}],"n").map(x=>x.n) }) + "\n");
