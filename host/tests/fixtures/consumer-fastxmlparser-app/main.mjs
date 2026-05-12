import { XMLParser, XMLBuilder } from "fast-xml-parser";

const xml = `<?xml version="1.0" encoding="UTF-8"?>
<root>
  <person id="1" age="30">
    <name>Ada</name>
    <email>ada@example.com</email>
  </person>
  <person id="2" age="45">
    <name>Bob</name>
  </person>
  <count>2</count>
  <ratio>0.5</ratio>
  <active>true</active>
</root>`;

const parser = new XMLParser({ ignoreAttributes: false, attributeNamePrefix: "@_" });
const obj = parser.parse(xml);

const builder = new XMLBuilder({ ignoreAttributes: false, attributeNamePrefix: "@_" });
const built = builder.build({ root: { count: 5, name: "test" } });

// Edge: empty element + CDATA
const xml2 = `<r><empty/><cdata><![CDATA[<raw>]]></cdata></r>`;
const obj2 = parser.parse(xml2);

process.stdout.write(JSON.stringify({
  rootCount: obj.root.count,
  firstPersonName: obj.root.person[0].name,
  firstPersonId: obj.root.person[0]["@_id"],
  ratio: obj.root.ratio,
  active: obj.root.active,
  builtHasName: built.includes("<name>test</name>"),
  emptyElement: obj2.r.empty,
  cdataText: obj2.r.cdata,
}) + "\n");
