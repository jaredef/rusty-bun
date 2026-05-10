// Middleware: validate request shape. Throws on missing fields.
export function validate(next) {
    return async function validatingHandler(ctx) {
        if (!ctx.payload) throw new Error("validate: payload required");
        if (typeof ctx.payload.user !== "string") throw new Error("validate: user required");
        if (typeof ctx.payload.action !== "string") throw new Error("validate: action required");
        return next(ctx);
    };
}
