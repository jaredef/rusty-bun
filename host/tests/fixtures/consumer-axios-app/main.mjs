import axios from "axios"; process.stdout.write(JSON.stringify({get:typeof axios.get,post:typeof axios.post,create:typeof axios.create,interceptors:typeof axios.interceptors}) + "\n");
