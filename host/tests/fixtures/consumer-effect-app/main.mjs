import { Effect, pipe } from "effect";

const program = pipe(
  Effect.succeed(5),
  Effect.map(n => n * 2),
  Effect.flatMap(n => Effect.succeed(n + 1)),
);

const result = await Effect.runPromise(program);

const failProg = pipe(
  Effect.fail("boom"),
  Effect.catchAll(e => Effect.succeed("recovered: " + e)),
);
const recovered = await Effect.runPromise(failProg);

const combined = await Effect.runPromise(
  Effect.all([Effect.succeed(1), Effect.succeed(2), Effect.succeed(3)])
);

process.stdout.write(JSON.stringify({ result, recovered, combined }) + "\n");
