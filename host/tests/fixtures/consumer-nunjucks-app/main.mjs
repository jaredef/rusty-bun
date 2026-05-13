import nunjucks from "nunjucks";
const r = nunjucks.renderString("Hello {{ name }}", { name: "Ada" });
process.stdout.write(JSON.stringify({ r }) + "\n");
