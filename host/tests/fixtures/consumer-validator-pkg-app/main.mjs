import validator from "validator";

process.stdout.write(JSON.stringify({
  email: validator.isEmail("a@b.com"),
  notEmail: validator.isEmail("not-email"),
  url: validator.isURL("https://example.com"),
  uuid: validator.isUUID("550e8400-e29b-41d4-a716-446655440000"),
  alphanumeric: validator.isAlphanumeric("abc123"),
  hexColor: validator.isHexColor("#ff0000"),
  jwt: validator.isJWT("eyJ.eyJ.sig"),
  ipv4: validator.isIP("192.168.1.1", 4),
  trim: validator.trim("  hello  "),
  normalize: validator.normalizeEmail("Foo@Bar.com"),
}) + "\n");
