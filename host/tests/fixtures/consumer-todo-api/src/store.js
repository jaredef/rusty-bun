// In-memory todo store. Exercises Map, structuredClone, and is a typical
// consumer of bare-specifier imports through node_modules.
import { nextId } from "idgen";

const todos = new Map();

export function createTodo(title) {
    const id = nextId();
    const todo = {
        id,
        title,
        done: false,
        createdAt: new Date(0),  // fixed for determinism
        tags: new Set(),
    };
    todos.set(id, todo);
    // Return a clone so callers can't mutate our state.
    return structuredClone(todo);
}

export function listTodos(filter) {
    const out = [];
    for (const t of todos.values()) {
        if (filter && filter.done !== undefined && t.done !== filter.done) continue;
        out.push(structuredClone(t));
    }
    return out;
}

export function markDone(id) {
    const t = todos.get(id);
    if (!t) return null;
    t.done = true;
    return structuredClone(t);
}

export function clear() {
    todos.clear();
}
