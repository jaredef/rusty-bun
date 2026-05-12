import { Parser, parseDocument } from "htmlparser2";

const lines = [];

// 1: event-based parser
{
  const events = [];
  const p = new Parser({
    onopentag: (n) => events.push("o:" + n),
    onclosetag: (n) => events.push("c:" + n),
    ontext: (t) => { if (t.trim()) events.push("t:" + t.trim()); },
  });
  p.write("<p>hello <b>world</b></p>");
  p.end();
  lines.push("1 " + events.join(","));
}

// 2: parseDocument → DOM tree
{
  const doc = parseDocument("<div><p>x</p><p>y</p></div>");
  const div = doc.children[0];
  lines.push("2 type=" + div.type + " tag=" + div.name + " kids=" + div.children.length);
}

// 3: attributes
{
  const doc = parseDocument('<a href="/test" class="link" data-id="42">go</a>');
  const a = doc.children[0];
  lines.push("3 href=" + a.attribs.href + " class=" + a.attribs.class + " id=" + a.attribs["data-id"]);
}

// 4: void / self-closing
{
  const events = [];
  const p = new Parser({
    onopentag: (n, a) => events.push("o:" + n + (a.src ? ":" + a.src : "")),
    onclosetag: (n) => events.push("c:" + n),
  });
  p.write('<img src="x.png"><br/>');
  p.end();
  lines.push("4 " + events.join(","));
}

// 5: text + entity decoding
{
  const doc = parseDocument("<p>hello&amp;world</p>");
  // text-node value reflects decoded entity
  lines.push("5 " + doc.children[0].children[0].data);
}

// 6: XML mode
{
  const events = [];
  const p = new Parser({
    onopentag: (n) => events.push("o:" + n),
    onclosetag: (n) => events.push("c:" + n),
  }, { xmlMode: true });
  p.write("<book><Title>A</Title></book>");
  p.end();
  lines.push("6 " + events.join(","));
}

process.stdout.write(lines.join("\n") + "\n");
