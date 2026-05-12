import * as yup from "yup";

const schema = yup.object({
  name: yup.string().required().min(1),
  age: yup.number().integer().min(0).max(150).required(),
  email: yup.string().email().optional(),
  tags: yup.array().of(yup.string()).default([]),
});

let goodValid = false, goodValue = null;
try {
  goodValue = await schema.validate({ name: "Ada", age: 32, email: "a@b.com", tags: ["x"] });
  goodValid = true;
} catch (_) {}

let badErrors = [];
try {
  await schema.validate({ name: "", age: -1, email: "not-email" }, { abortEarly: false });
} catch (e) {
  badErrors = (e.errors || []).map(String).sort();
}

const cast = schema.cast({ name: "Bob", age: "42" });

process.stdout.write(JSON.stringify({
  goodValid,
  goodName: goodValue && goodValue.name,
  badErrorCount: badErrors.length,
  badErrorsHaveAge: badErrors.some(e => /age/i.test(e)),
  badErrorsHaveName: badErrors.some(e => /name/i.test(e)),
  badErrorsHaveEmail: badErrors.some(e => /email/i.test(e)),
  castAgeType: typeof cast.age,
}) + "\n");
