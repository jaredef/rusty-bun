// Simulated async data sources. Real consumer code would call upstream
// APIs; here we return deterministic values via Promises so the
// differential is byte-identical between runtimes.

export function fetchUser(id) {
    return Promise.resolve({ id, name: "user-" + id });
}

export function fetchProfile(id) {
    return Promise.resolve({ id, bio: "bio for " + id });
}

export function fetchSettings(id) {
    return Promise.resolve({ id, theme: id % 2n === 0n ? "dark" : "light" });
}

// A source that fails sometimes — for Promise.allSettled exercise.
export function fetchOptional(id) {
    if (id === 3n) return Promise.reject(new Error("missing for " + id));
    return Promise.resolve({ id, optional: "opt-" + id });
}

// A slow source — for Promise.race timeout pattern.
export function slowFetch(id, ms) {
    return new Promise((resolve) => {
        setTimeout(() => resolve({ id, slow: true }), ms);
    });
}
