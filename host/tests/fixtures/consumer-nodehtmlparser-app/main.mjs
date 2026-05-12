import { parse } from "node-html-parser";

const root = parse(`
<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
  <h1 class="hdr">Hello</h1>
  <ul id="items">
    <li class="item">one</li>
    <li class="item special">two</li>
    <li class="item">three</li>
  </ul>
  <a href="https://example.com">link</a>
</body>
</html>
`);

const out = {
  title: root.querySelector("title").text,
  hdr: root.querySelector("h1.hdr").text,
  itemCount: root.querySelectorAll("li.item").length,
  specialText: root.querySelector("li.special").text,
  href: root.querySelector("a").getAttribute("href"),
  textOfItems: root.querySelectorAll("li.item").map(el => el.text),
  hasClass: root.querySelector("li.special").classList.contains("item"),
};

// Mutation
root.querySelector("h1").set_content("Goodbye");

out.afterMutation = {
  hdr: root.querySelector("h1").text,
};

process.stdout.write(JSON.stringify(out) + "\n");
