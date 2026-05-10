// Middleware: augment context with derived fields.
export function augment(next) {
    return async function augmentingHandler(ctx) {
        ctx.normalized = {
            user: ctx.payload.user.toLowerCase().trim(),
            action: ctx.payload.action.toUpperCase().trim(),
            timestamp: 0,  // fixed for deterministic differential
        };
        return next(ctx);
    };
}
