import Joi from "joi";
const s = Joi.object({ name: Joi.string().required() });
const r = s.validate({ name: "x" });
process.stdout.write(JSON.stringify({ hasJoi: typeof Joi, valid: r.error === undefined }) + "\n");
