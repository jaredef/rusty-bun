import Ajv from "ajv";

const ajv = new Ajv();

const schema = {
  type: "object",
  properties: {
    name: { type: "string", minLength: 1 },
    age: { type: "integer", minimum: 0, maximum: 150 },
    tags: { type: "array", items: { type: "string" }, uniqueItems: true },
  },
  required: ["name", "age"],
  additionalProperties: false,
};

const validate = ajv.compile(schema);

const good = validate({ name: "Ada", age: 32, tags: ["math", "logic"] });
const bad = validate({ name: "", age: -1, extra: "nope" });
const badErrors = (validate.errors || []).map(e => e.keyword).sort();

const missing = validate({ name: "Bob" });
const missingErrors = (validate.errors || []).map(e => e.keyword).sort();

process.stdout.write(JSON.stringify({
  goodValid: good,
  badValid: bad,
  badErrors,
  missingValid: missing,
  missingErrors,
}) + "\n");
