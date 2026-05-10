// Middleware composition: right-to-left chain into a single handler.
// Real frameworks (Hono, Koa, Express) all expose a similar shape.
export function compose(...middlewares) {
    return function composed(terminal) {
        return middlewares.reduceRight((next, mw) => mw(next), terminal);
    };
}
