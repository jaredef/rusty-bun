const cases = {
  // 1. Simple scalars
  scalars: Bun.YAML.parse(`
name: rusty-bun
count: 42
ratio: 3.14
ok: true
miss: null
empty:
neg: -5
`),
  // 2. Flow style
  flow: Bun.YAML.parse(`items: [1, 2, "three"]
config: {host: localhost, port: 8080}`),
  // 3. Block list
  list: Bun.YAML.parse(`
fruits:
  - apple
  - banana
  - cherry
`),
  // 4. Nested
  nested: Bun.YAML.parse(`
server:
  host: 0.0.0.0
  port: 3000
  routes:
    - /
    - /api
db:
  user: admin
  pass: "s3cret"
`),
  // 5. Quoted strings with special chars
  quoted: Bun.YAML.parse(`
double: "a:b#c"
single: 'it''s ok'
plain: hello world
`),
  // 6. Block scalar (literal)
  literal: Bun.YAML.parse(`
script: |
  echo hi
  exit 0
`),
  // 7. Comments + blank lines
  commented: Bun.YAML.parse(`
# top-level comment
a: 1   # trailing
b: 2

c: 3
`),
};

// Stringify round-trips for a few
const roundTrip = Bun.YAML.parse(Bun.YAML.stringify({
  name: "test",
  nested: { a: 1, b: [1, 2, 3] },
  flag: true,
}));

process.stdout.write(JSON.stringify({ cases, roundTrip }) + "\n");
