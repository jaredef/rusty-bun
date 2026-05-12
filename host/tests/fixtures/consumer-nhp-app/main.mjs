import { parse } from "node-html-parser";

const lines = [];

// 1: parse + querySelector
{
  const root = parse('<div><p id="a">Hello</p><p class="b">World</p></div>');
  lines.push("1 firstP=" + root.querySelector("p").text +
             " byId=" + root.querySelector("#a").text +
             " byClass=" + root.querySelector(".b").text);
}

// 2: all
{
  const root = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
  const items = root.querySelectorAll("li").map(li => li.text);
  lines.push("2 " + JSON.stringify(items));
}

// 3: attributes
{
  const root = parse('<a href="/x" data-id="42">link</a>');
  const a = root.querySelector("a");
  lines.push("3 href=" + a.getAttribute("href") + " data=" + a.getAttribute("data-id"));
}

// 4: nested + getElementsByTagName
{
  const root = parse("<article><h1>title</h1><p>body1</p><p>body2</p></article>");
  const ps = root.getElementsByTagName("p").map(p => p.text);
  lines.push("4 " + JSON.stringify(ps));
}

// 5: toString round-trip
{
  const root = parse("<div><b>hi</b></div>");
  lines.push("5 " + root.toString());
}

process.stdout.write(lines.join("\n") + "\n");
