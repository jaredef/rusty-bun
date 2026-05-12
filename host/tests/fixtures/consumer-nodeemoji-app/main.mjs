import * as emoji from "node-emoji";

const out = {
  heart: emoji.get("heart"),
  smile: emoji.get(":smile:"),
  emojify: emoji.emojify("I :heart: :coffee:!"),
  has: emoji.has("heart"),
  hasGarbage: emoji.has("notarealemoji"),
  unemojify: emoji.unemojify("I ❤️ ☕!"),
  find: emoji.find("❤️"),
};
process.stdout.write(JSON.stringify(out) + "\n");
