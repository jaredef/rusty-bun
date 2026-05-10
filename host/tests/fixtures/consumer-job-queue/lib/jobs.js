// Class inheritance hierarchy: BaseJob → Job → PriorityJob.
// Real consumer code in OO-flavored frameworks uses this shape pervasively.

import { randomUUID } from "node:crypto";
import { InvalidJobError } from "./errors.js";

// Symbol-keyed private state — internal metadata that shouldn't be
// enumerable in serialization.
const kInternal = Symbol("internal");

export class BaseJob {
    constructor(kind) {
        if (!kind) throw new InvalidJobError("job kind required", null);
        this.id = randomUUID();
        this.kind = kind;
        this.createdAt = new Date(0);  // fixed for determinism
        this[kInternal] = { startedAt: null, completedAt: null };
    }
    markStarted() { this[kInternal].startedAt = 1; }
    markCompleted() { this[kInternal].completedAt = 2; }
    isCompleted() { return this[kInternal].completedAt !== null; }
}

export class Job extends BaseJob {
    constructor(kind, payload) {
        super(kind);
        this.payload = payload || {};
    }
    summary() {
        return this.kind + ":" + JSON.stringify(this.payload);
    }
}

export class PriorityJob extends Job {
    constructor(kind, payload, priority) {
        super(kind, payload);
        this.priority = typeof priority === "number" ? priority : 0;
    }
    compareTo(other) {
        // Higher priority first; ties broken by createdAt.
        if (this.priority !== other.priority) return other.priority - this.priority;
        return this.createdAt.getTime() - other.createdAt.getTime();
    }
}

// Symbol export so callers can opt into reading the internal slot via
// e.g. JSON replacer filtering. Real consumer code often exports symbols.
export { kInternal };
