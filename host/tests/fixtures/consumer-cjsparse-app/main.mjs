// commander ^14 — CLI parser (distinct axis from yargs/arg).
import { Command } from "commander";

const lines = [];

const program = new Command();
program
  .name("test")
  .option("-v, --verbose", "verbose mode")
  .option("-n, --name <name>", "name option")
  .option("-c, --count <n>", "count", v => parseInt(v, 10), 1)
  .action(() => {});

program.parse(["node", "test", "-v", "-n", "alice", "-c", "3"], { from: "node" });
const opts = program.opts();
lines.push("1 v=" + opts.verbose + " n=" + opts.name + " c=" + opts.count);

const program2 = new Command();
program2
  .argument("<file>")
  .argument("[mode]", "mode", "default")
  .action((file, mode) => {
    lines.push("2 file=" + file + " mode=" + mode);
  });
program2.parse(["node", "test", "input.txt"], { from: "node" });

const program3 = new Command();
program3.command("foo").option("-x").action(function () { lines.push("3 sub=foo x=" + !!this.opts().x); });
program3.parse(["node", "test", "foo", "-x"], { from: "node" });

const program4 = new Command();
program4.option("-l, --list <items>", "list", v => v.split(","));
program4.parse(["node", "test", "-l", "a,b,c"], { from: "node" });
lines.push("4 list=" + JSON.stringify(program4.opts().list));

const program5 = new Command();
program5.version("1.2.3");
lines.push("5 v=" + program5.version());

const program6 = new Command();
program6.option("--no-color", "disable color");
program6.parse(["node", "test", "--no-color"], { from: "node" });
lines.push("6 color=" + program6.opts().color);

process.stdout.write(lines.join("\n") + "\n");
