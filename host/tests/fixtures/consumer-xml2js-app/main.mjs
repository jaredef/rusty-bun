import { parseString, Builder } from "xml2js";

const xml = `<?xml version="1.0"?>
<root>
  <person id="1"><name>Ada</name><age>32</age></person>
  <person id="2"><name>Bob</name><age>45</age></person>
</root>`;

const parsed = await new Promise((res, rej) => {
  parseString(xml, (err, result) => err ? rej(err) : res(result));
});

const builder = new Builder({ headless: true });
const built = builder.buildObject({ root: { item: "hello" } });

process.stdout.write(JSON.stringify({
  rootKeys: Object.keys(parsed.root).sort(),
  firstPersonName: parsed.root.person[0].name[0],
  firstPersonId: parsed.root.person[0].$.id,
  hasBuilt: typeof built === "string",
  builtHasItem: built.includes("<item>hello</item>"),
}) + "\n");
