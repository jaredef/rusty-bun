// Tier-J consumer #18: modern class features (spec-first M9 / M9.bis).
//
// In-basin axes not yet exercised:
//   - Private class fields (#field) — ES2022
//   - Static class fields and methods
//   - super in subclass constructor + method override
//   - error.cause (ES2022 chained-error syntax)
//   - Promise.any (ES2021 any-of, opposite of allSettled)
//   - new.target meta-property for abstract-class enforcement
//   - Class with static block (ES2022)
//   - Optional chaining + nullish coalescing in class methods

// Abstract base class — uses new.target to forbid direct instantiation.
class StateMachine {
    #state;
    #transitions;
    static DEFAULT_INITIAL = "idle";
    static #instanceCount = 0;
    static {
        // Static initialization block (ES2022).
        StateMachine.classVersion = "1.0";
    }

    constructor(transitions, initial) {
        if (new.target === StateMachine) {
            throw new TypeError("StateMachine is abstract; subclass it");
        }
        this.#state = initial ?? StateMachine.DEFAULT_INITIAL;
        this.#transitions = transitions;
        StateMachine.#instanceCount++;
    }

    get state() { return this.#state; }

    static get instanceCount() { return StateMachine.#instanceCount; }
    static resetCount() { StateMachine.#instanceCount = 0; }

    transition(event) {
        const next = this.#transitions[this.#state]?.[event];
        if (!next) {
            throw new Error(
                `no transition from ${this.#state} on ${event}`,
                { cause: { state: this.#state, event } }
            );
        }
        this.#state = next;
        return next;
    }

    canTransition(event) {
        return Boolean(this.#transitions[this.#state]?.[event]);
    }
}

// Concrete subclass.
class TrafficLight extends StateMachine {
    #cycles = 0;
    constructor() {
        super({
            red:    { TIMER: "green" },
            green:  { TIMER: "yellow" },
            yellow: { TIMER: "red" },
        }, "red");
    }

    tick() {
        const before = this.state;
        const after = this.transition("TIMER");
        if (after === "red" && before === "yellow") this.#cycles++;
        return after;
    }

    get cycles() { return this.#cycles; }
}

async function selfTest() {
    const results = [];

    StateMachine.resetCount();

    // 1. Abstract-class enforcement via new.target.
    let abstractCaught = null;
    try { new StateMachine({}, "x"); }
    catch (e) { abstractCaught = e; }
    results.push(["abstract-newtarget",
        abstractCaught instanceof TypeError &&
        /abstract/.test(abstractCaught.message)]);

    // 2. Static block ran during class definition.
    results.push(["static-block", StateMachine.classVersion === "1.0"]);

    // 3. Subclass instantiation succeeds.
    const tl = new TrafficLight();
    results.push(["subclass-construct",
        tl.state === "red" && tl instanceof StateMachine]);

    // 4. Private field initialization in subclass.
    results.push(["subclass-private-field", tl.cycles === 0]);

    // 5. Transition cycle.
    tl.tick();  // red → green
    tl.tick();  // green → yellow
    tl.tick();  // yellow → red, cycles++
    results.push(["transition-cycle",
        tl.state === "red" && tl.cycles === 1]);

    // 6. Bad transition throws Error with .cause.
    const bad = new TrafficLight();
    let causedErr = null;
    try { bad.transition("EXPLODE"); }
    catch (e) { causedErr = e; }
    results.push(["error-cause",
        causedErr instanceof Error &&
        causedErr.cause &&
        causedErr.cause.state === "red" &&
        causedErr.cause.event === "EXPLODE"]);

    // 7. Static method increments instance counter via private static field.
    results.push(["static-private-field",
        StateMachine.instanceCount === 2]);  // tl and bad

    // 8. Optional chaining + nullish coalescing.
    const tl2 = new TrafficLight();
    const canFire = tl2.canTransition("TIMER");
    const canNotFire = tl2.canTransition("UNKNOWN");
    results.push(["optional-chain-nullish",
        canFire === true && canNotFire === false]);

    // 9. Promise.any — first to resolve wins; rejections are ignored unless
    // all reject.
    const winner = await Promise.any([
        Promise.reject(new Error("slow")),
        Promise.resolve("fast"),
        new Promise((r) => setTimeout(() => r("delayed"), 100)),
    ]);
    results.push(["promise-any", winner === "fast"]);

    // 10. Promise.any all-rejected → AggregateError.
    let aggregateErr = null;
    try {
        await Promise.any([
            Promise.reject(new Error("a")),
            Promise.reject(new Error("b")),
        ]);
    } catch (e) { aggregateErr = e; }
    results.push(["promise-any-all-reject",
        aggregateErr instanceof AggregateError &&
        Array.isArray(aggregateErr.errors) &&
        aggregateErr.errors.length === 2]);

    // 11. Subclass can access inherited public methods (canTransition).
    const tl3 = new TrafficLight();
    results.push(["inherited-public",
        typeof tl3.canTransition === "function" &&
        tl3.canTransition("TIMER") === true]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
