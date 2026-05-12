// change-case ^5 — unified case-conversion suite.
import { camelCase, pascalCase, kebabCase, sentenceCase, capitalCase, constantCase } from "change-case";

const lines = [];
lines.push("1 " + camelCase("hello world"));
lines.push("2 " + pascalCase("hello world"));
lines.push("3 " + kebabCase("HelloWorld"));
lines.push("4 " + sentenceCase("HELLO_WORLD"));
lines.push("5 " + capitalCase("hello-world"));
lines.push("6 " + constantCase("HelloWorldFoo"));

process.stdout.write(lines.join("\n") + "\n");
