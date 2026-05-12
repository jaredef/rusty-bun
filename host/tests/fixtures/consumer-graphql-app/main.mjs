import { parse, print, validate, buildSchema, graphql } from "graphql";

const schema = buildSchema(`
  type User {
    id: ID!
    name: String!
    age: Int
  }
  type Query {
    user(id: ID!): User
    users: [User!]!
  }
`);

const root = {
  user: ({ id }) => ({ id, name: "Ada", age: 32 }),
  users: () => [{ id: "1", name: "Ada", age: 32 }, { id: "2", name: "Bob", age: 45 }],
};

const query = `{ user(id: "1") { id name age } }`;
const result = await graphql({ schema, source: query, rootValue: root });

const query2 = `{ users { id name } }`;
const result2 = await graphql({ schema, source: query2, rootValue: root });

const parsed = parse(query);
const printed = print(parsed);

process.stdout.write(JSON.stringify({
  user: result.data,
  usersCount: result2.data.users.length,
  parsedType: parsed.kind,
  parsedDefCount: parsed.definitions.length,
  printedHasFields: printed.includes("id") && printed.includes("name"),
}) + "\n");
