import { Command, CommanderError } from "commander";

const program = new Command();
program
  .name("test-cli")
  .description("test")
  .version("1.0.0")
  .option("-n, --name <name>", "name")
  .option("-c, --count <n>", "count", parseInt, 1)
  .action(() => {});

program.exitOverride();
program.parse(["node", "test-cli", "--name", "ada", "--count", "5"]);

const out = {
  name: program.opts().name,
  count: program.opts().count,
  hasCommandClass: typeof Command === "function",
  hasErrorClass: typeof CommanderError === "function",
  programName: program.name(),
};

let errInfo = null;
try {
  const p2 = new Command();
  p2.exitOverride();
  p2.option("-r, --required <v>", "required").action(() => {});
  p2.parse(["node", "test", "--unknown"], { from: "node" });
} catch (e) {
  errInfo = {
    name: e && e.name,
    isErr: e instanceof Error,
    isCmdErr: e instanceof CommanderError,
    code: e && e.code,
  };
}
out.errInfo = errInfo;

process.stdout.write(JSON.stringify(out) + "\n");
