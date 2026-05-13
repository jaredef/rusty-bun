import PQueue from "p-queue"; const q=new PQueue({concurrency:2}); process.stdout.write(JSON.stringify({size:q.size,pending:q.pending,hasAdd:typeof q.add}) + "\n");
