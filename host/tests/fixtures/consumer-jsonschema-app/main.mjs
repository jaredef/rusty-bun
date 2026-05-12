import { Validator } from "jsonschema";

const v = new Validator();
const schema = {
  type: "object",
  properties: {
    name: { type: "string", minLength: 1 },
    age: { type: "integer", minimum: 0, maximum: 150 },
    tags: { type: "array", items: { type: "string" } },
  },
  required: ["name", "age"],
};

const good = v.validate({ name: "Ada", age: 32, tags: ["math", "logic"] }, schema);
const badType = v.validate({ name: "Ada", age: "thirty" }, schema);
const missing = v.validate({ name: "Ada" }, schema);
const outOfRange = v.validate({ name: "Ada", age: 999 }, schema);

process.stdout.write(JSON.stringify({
  goodValid: good.valid,
  goodErrors: good.errors.length,
  badTypeValid: badType.valid,
  badTypeErr0: badType.errors[0] && badType.errors[0].name,
  missingValid: missing.valid,
  missingErr0: missing.errors[0] && missing.errors[0].name,
  outOfRangeValid: outOfRange.valid,
  outOfRangeErr0: outOfRange.errors[0] && outOfRange.errors[0].name,
}) + "\n");
