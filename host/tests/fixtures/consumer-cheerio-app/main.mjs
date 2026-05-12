import * as cheerio from "cheerio";

const html = `
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
`;

const $ = cheerio.load(html);

const out = {
  title: $("title").text(),
  hdr: $("h1.hdr").text(),
  itemCount: $("li.item").length,
  specialText: $("li.special").text(),
  href: $("a").attr("href"),
  textOfItems: $("li.item").map((i, el) => $(el).text()).get(),
  hasClass: $("li.special").hasClass("item"),
};

$("h1").text("Goodbye");
$("ul").append("<li class=\"item\">four</li>");

out.afterMutation = {
  hdr: $("h1").text(),
  itemCount: $("li.item").length,
};

process.stdout.write(JSON.stringify(out) + "\n");
