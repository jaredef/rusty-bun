import { camelCase, snakeCase, kebabCase } from "change-case";
process.stdout.write(JSON.stringify({ c: camelCase("hello world"), s: snakeCase("HelloWorld"), k: kebabCase("HelloWorld") }) + "\n");
