import MarkdownIt from "markdown-it";
const md = new MarkdownIt();
process.stdout.write(JSON.stringify({ html: md.render("# Hello").trim() }) + "\n");
