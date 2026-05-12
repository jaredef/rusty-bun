// xstate ^5 — state machine library. Distinct paradigm (finite-state
// machines / statecharts). Tests createMachine + createActor + state
// transitions + context updates.
import { createMachine, createActor, assign } from "xstate";

const lines = [];

// 1: basic toggle machine
{
  const machine = createMachine({
    id: "toggle",
    initial: "off",
    states: {
      off: { on: { TOGGLE: "on" } },
      on: { on: { TOGGLE: "off" } },
    },
  });
  const actor = createActor(machine).start();
  const s0 = actor.getSnapshot().value;
  actor.send({ type: "TOGGLE" });
  const s1 = actor.getSnapshot().value;
  actor.send({ type: "TOGGLE" });
  const s2 = actor.getSnapshot().value;
  lines.push("1 " + s0 + "->" + s1 + "->" + s2);
}

// 2: machine with context + assign action
{
  const counter = createMachine({
    id: "counter",
    context: { count: 0 },
    on: {
      INC: { actions: assign({ count: ({ context }) => context.count + 1 }) },
      DEC: { actions: assign({ count: ({ context }) => context.count - 1 }) },
    },
  });
  const actor = createActor(counter).start();
  actor.send({ type: "INC" });
  actor.send({ type: "INC" });
  actor.send({ type: "INC" });
  actor.send({ type: "DEC" });
  lines.push("2 count=" + actor.getSnapshot().context.count);
}

// 3: nested states
{
  const trafficLight = createMachine({
    id: "traffic",
    initial: "red",
    states: {
      red: { on: { TIMER: "green" } },
      green: { on: { TIMER: "yellow" } },
      yellow: { on: { TIMER: "red" } },
    },
  });
  const actor = createActor(trafficLight).start();
  const seq = [actor.getSnapshot().value];
  for (let i = 0; i < 4; i++) {
    actor.send({ type: "TIMER" });
    seq.push(actor.getSnapshot().value);
  }
  lines.push("3 " + seq.join("->"));
}

// 4: guarded transition
{
  const door = createMachine({
    id: "door",
    initial: "closed",
    context: { hasKey: false },
    states: {
      closed: {
        on: {
          OPEN: [
            { target: "open", guard: ({ context }) => context.hasKey },
            { target: "stuck" },
          ],
          KEY: { actions: assign({ hasKey: true }) },
        },
      },
      open: {},
      stuck: {},
    },
  });
  const actor = createActor(door).start();
  actor.send({ type: "OPEN" });
  const s1 = actor.getSnapshot().value;
  // restart
  const actor2 = createActor(door).start();
  actor2.send({ type: "KEY" });
  actor2.send({ type: "OPEN" });
  const s2 = actor2.getSnapshot().value;
  lines.push("4 no-key=" + s1 + " key=" + s2);
}

// 5: status field
{
  const m = createMachine({ id: "m", initial: "a", states: { a: { type: "final" } } });
  const actor = createActor(m).start();
  lines.push("5 status=" + actor.getSnapshot().status);
}

process.stdout.write(lines.join("\n") + "\n");
