// PriorityQueue with an async-generator drain. Exercises both the
// imperative push/pop API and the for-await-of iteration shape.

import { QueueClosedError } from "./errors.js";

export class PriorityQueue {
    constructor() {
        this._items = [];
        this._closed = false;
    }
    enqueue(job) {
        if (this._closed) throw new QueueClosedError();
        this._items.push(job);
        // Maintain priority order; small queue → sort is fine.
        this._items.sort((a, b) => a.compareTo(b));
    }
    size() { return this._items.length; }
    close() { this._closed = true; }
    closed() { return this._closed; }
    // Imperative drain.
    dequeue() {
        return this._items.shift();
    }
    // Async-generator drain. yields each job in priority order, marking
    // started/completed around the yield. Real consumer code uses this
    // shape for streaming-style workers.
    async *drain() {
        while (this._items.length > 0) {
            const job = this._items.shift();
            job.markStarted();
            yield job;
            job.markCompleted();
        }
    }
}
