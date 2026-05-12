import unraw from "unraw";

const cases = [
  String.raw`hello\nworld`,
  String.raw`tab\there`,
  String.raw`\x41\x42\x43`,
  String.raw`Aé`,
  String.raw`\u{1F600}`,
  String.raw`quote\"end`,
  String.raw`back\\slash`,
];

const out = cases.map(c => unraw(c));
process.stdout.write(JSON.stringify(out) + "\n");
