import ky from "ky"; process.stdout.write(JSON.stringify({get:typeof ky.get,post:typeof ky.post,create:typeof ky.create}) + "\n");
