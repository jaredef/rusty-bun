import mime from "mime-types";

process.stdout.write(JSON.stringify({
  jsType: mime.lookup("script.js"),
  htmlType: mime.lookup("page.html"),
  jsonType: mime.lookup("data.json"),
  pngType: mime.lookup("img.png"),
  unknownType: mime.lookup("unknown.qqq"),
  jsExtension: mime.extension("application/javascript"),
  htmlExtension: mime.extension("text/html"),
  charsetJs: mime.charset("application/javascript"),
  charsetHtml: mime.charset("text/html"),
}) + "\n");
