import gql from "graphql-tag";
const q = gql`query { hello }`;
process.stdout.write(JSON.stringify({ kind: q.kind, def: q.definitions[0].operation }) + "\n");
