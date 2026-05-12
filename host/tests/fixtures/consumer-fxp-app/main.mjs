import { XMLParser, XMLBuilder } from "fast-xml-parser";

const lines = [];

// 1: basic parse
{
  const xml = "<root><a>1</a><b>2</b></root>";
  const obj = new XMLParser().parse(xml);
  lines.push("1 " + JSON.stringify(obj));
}

// 2: attributes
{
  const xml = '<root id="1" name="test"><child kind="a">x</child></root>';
  const obj = new XMLParser({ ignoreAttributes: false }).parse(xml);
  lines.push("2 " + JSON.stringify(obj));
}

// 3: nested + array
{
  const xml = "<list><item>a</item><item>b</item><item>c</item></list>";
  const obj = new XMLParser({ isArray: () => true }).parse(xml);
  lines.push("3 " + JSON.stringify(obj));
}

// 4: build from object
{
  const builder = new XMLBuilder();
  const out = builder.build({ root: { a: 1, b: 2 } }).trim();
  lines.push("4 " + out);
}

// 5: round-trip parse → build
{
  const xml = "<doc><h>title</h><b>body</b></doc>";
  const obj = new XMLParser().parse(xml);
  const back = new XMLBuilder().build(obj).trim();
  lines.push("5 roundTripEq=" + (back === xml));
}

// 6: CDATA
{
  const xml = "<note><body><![CDATA[<b>raw</b>]]></body></note>";
  const obj = new XMLParser().parse(xml);
  lines.push("6 body=" + JSON.stringify(obj.note.body));
}

process.stdout.write(lines.join("\n") + "\n");
