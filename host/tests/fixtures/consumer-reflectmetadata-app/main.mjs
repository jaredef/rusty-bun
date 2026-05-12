import "reflect-metadata";

const target = {};
Reflect.defineMetadata("design:type", String, target, "prop");
Reflect.defineMetadata("custom:val", { hello: "world" }, target, "prop");

const out = {
  hasGet: typeof Reflect.getMetadata === "function",
  hasDefine: typeof Reflect.defineMetadata === "function",
  hasHas: typeof Reflect.hasMetadata === "function",
  designType: Reflect.getMetadata("design:type", target, "prop") === String,
  customVal: Reflect.getMetadata("custom:val", target, "prop"),
  keys: Reflect.getMetadataKeys(target, "prop").sort(),
  has: Reflect.hasMetadata("design:type", target, "prop"),
  hasMissing: Reflect.hasMetadata("nope", target, "prop"),
};
process.stdout.write(JSON.stringify(out) + "\n");
