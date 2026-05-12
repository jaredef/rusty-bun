import pluralize from "pluralize";

const lines = [];

// 1: basic
lines.push("1 cat=" + pluralize("cat") + " mouse=" + pluralize("mouse") + " sheep=" + pluralize("sheep"));

// 2: with count
lines.push("2 " + pluralize("dog", 1) + " " + pluralize("dog", 2) + " " + pluralize("dog", 0));

// 3: singular
lines.push("3 " + pluralize.singular("cats") + " " + pluralize.singular("mice"));

// 4: isPlural / isSingular
lines.push("4 " + pluralize.isPlural("dogs") + " " + pluralize.isSingular("dog") + " " + pluralize.isPlural("dog"));

// 5: count helper
lines.push("5 " + pluralize("dog", 1, true) + " " + pluralize("dog", 5, true));

// 6: irregular
lines.push("6 person=" + pluralize("person") + " child=" + pluralize("child"));

process.stdout.write(lines.join("\n") + "\n");
