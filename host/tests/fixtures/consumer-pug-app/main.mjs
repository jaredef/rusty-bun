import pug from "pug";

const t1 = pug.render("p Hello, #{name}!", { name: "world" });
const t2 = pug.render(`
ul
  each item in items
    li= item
`, { items: ["a", "b", "c"] });

const t3 = pug.render(`
- var n = 3
if n > 2
  p big
else
  p small
`);

const compile = pug.compile("h1= title");
const t4 = compile({ title: "Hello" });

process.stdout.write(JSON.stringify({ t1, t2, t3, t4 }) + "\n");
