// Custom Error subclasses — common consumer pattern for typed errors.
export class InvalidJobError extends Error {
    constructor(message, jobId) {
        super(message);
        this.name = "InvalidJobError";
        this.jobId = jobId;
    }
}

export class QueueClosedError extends Error {
    constructor() {
        super("queue is closed");
        this.name = "QueueClosedError";
    }
}
