import arg from "arg";

const lines = [];

// 1: basic flags + positional
{
  const args = arg({
    "--verbose": Boolean,
    "--name": String,
    "--count": Number,
    "-v": "--verbose",
  }, { argv: ["-v", "--name", "alice", "--count", "5", "input.txt"] });
  lines.push("1 verbose=" + args["--verbose"] + " name=" + args["--name"] + " count=" + args["--count"] + " rest=" + JSON.stringify(args._));
}

// 2: equals form
{
  const args = arg({ "--key": String }, { argv: ["--key=value"] });
  lines.push("2 key=" + args["--key"]);
}

// 3: array (repeated)
{
  const args = arg({ "--tag": [String] }, { argv: ["--tag", "a", "--tag", "b", "--tag", "c"] });
  lines.push("3 " + JSON.stringify(args["--tag"]));
}

// 4: stop at -- separator
{
  const args = arg({ "--foo": String }, { argv: ["--foo", "bar", "--", "--not-a-flag", "x"] });
  lines.push("4 foo=" + args["--foo"] + " rest=" + JSON.stringify(args._));
}

// 5: unknown arg throws (when permissive:false default)
{
  try {
    arg({ "--known": String }, { argv: ["--unknown"] });
    lines.push("5 NOT_THROWN");
  } catch (e) {
    lines.push("5 threw=" + (e instanceof Error) + " code=" + e.code);
  }
}

// 6: permissive
{
  const args = arg({ "--known": String }, { argv: ["--unknown", "--known", "v"], permissive: true });
  lines.push("6 known=" + args["--known"] + " rest=" + JSON.stringify(args._));
}

process.stdout.write(lines.join("\n") + "\n");
