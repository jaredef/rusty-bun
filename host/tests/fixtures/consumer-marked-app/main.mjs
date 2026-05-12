// marked ^14 — Markdown parser. Tests tokenization + HTML emission +
// configurable renderer + lexer access.
import { marked, Lexer } from "marked";

const lines = [];

// 1: basic parse
{
  const md = "# Hello\n\nWorld";
  const html = marked.parse(md).trim();
  lines.push("1 " + html.replace(/\n/g, " | "));
}

// 2: bold / italic / code
{
  const html = marked.parse("**bold** _italic_ `code`").trim();
  lines.push("2 " + html.replace(/\n/g, " | "));
}

// 3: link
{
  const html = marked.parse("[google](https://google.com)").trim();
  lines.push("3 " + html.replace(/\n/g, " | "));
}

// 4: list + nested
{
  const md = "- a\n- b\n  - c\n- d";
  const html = marked.parse(md).trim();
  lines.push("4 " + html.replace(/\n/g, " | "));
}

// 5: code fence
{
  const md = "```js\nconst x = 1;\n```";
  const html = marked.parse(md).trim();
  lines.push("5 " + html.replace(/\n/g, " | "));
}

// 6: lexer tokens
{
  const lexer = new Lexer();
  const tokens = lexer.lex("# heading\n\nparagraph");
  lines.push("6 types=" + tokens.map(t => t.type).join(","));
}

// 7: blockquote
{
  const html = marked.parse("> quoted text").trim();
  lines.push("7 " + html.replace(/\n/g, " | "));
}

process.stdout.write(lines.join("\n") + "\n");
