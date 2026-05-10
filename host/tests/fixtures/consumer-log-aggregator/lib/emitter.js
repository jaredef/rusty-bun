// User-defined event-emitter. Real Bun consumers either use this shape
// or import from "node:events" — keeping it user-defined makes the
// fixture portable without depending on a node:events wiring.

export class Emitter {
    constructor() {
        this._listeners = new Map();  // type → array of listeners
    }
    on(type, fn) {
        let list = this._listeners.get(type);
        if (!list) { list = []; this._listeners.set(type, list); }
        list.push(fn);
        return this;
    }
    off(type, fn) {
        const list = this._listeners.get(type);
        if (!list) return this;
        const idx = list.indexOf(fn);
        if (idx >= 0) list.splice(idx, 1);
        return this;
    }
    emit(type, payload) {
        const list = this._listeners.get(type);
        if (!list) return false;
        // Snapshot listeners + clone payload so subscribers can't mutate
        // each other or the source.
        const snapshot = list.slice();
        const cloned = structuredClone(payload);
        for (const fn of snapshot) fn(cloned);
        return true;
    }
    listenerCount(type) {
        const list = this._listeners.get(type);
        return list ? list.length : 0;
    }
}
