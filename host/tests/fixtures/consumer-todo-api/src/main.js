// Tier-J consumer-shape pilot. A tiny Bun-flavored todo API.
//
// Bun-portable: uses Bun.serve fetch-handler dispatch only. Route-table
// matching is done in-handler with URL+URLSearchParams, since Bun's
// server.fetch() bypasses the routes table (route table fires only
// through actual HTTP transport in Bun, per M8 reconciliation 2026-05-10).
//
// Body methods are awaited per WHATWG.

import { createTodo, listTodos, markDone, clear } from "./store.js";

function jsonResponse(body, status) {
    return new Response(JSON.stringify(body), {
        status: status || 200,
        headers: { "content-type": "application/json" },
    });
}

const server = Bun.serve({
    port: 0,  // ephemeral; Bun-portable
    async fetch(req) {
        const url = new URL(req.url);
        const path = url.pathname;
        const method = req.method;

        if (path === "/health" && method === "GET") {
            return new Response("ok");
        }

        if (path === "/todos" && method === "GET") {
            const doneParam = url.searchParams.get("done");
            const filter = doneParam === null ? undefined : { done: doneParam === "true" };
            return jsonResponse(listTodos(filter));
        }

        if (path === "/todos" && method === "POST") {
            const body = await req.json();
            if (!body.title) return jsonResponse({ error: "title required" }, 400);
            return jsonResponse(createTodo(body.title), 201);
        }

        const doneMatch = path.match(/^\/todos\/([^\/]+)\/done$/);
        if (doneMatch && method === "POST") {
            const id = doneMatch[1];
            const t = markDone(id);
            if (!t) return jsonResponse({ error: "not found" }, 404);
            return jsonResponse(t);
        }

        return new Response("Not Found", { status: 404 });
    },
});

async function runSelfTest() {
    clear();
    const results = [];

    const health = await server.fetch(new Request("http://localhost:4000/health"));
    results.push(["health", health.status === 200 && (await health.text()) === "ok"]);

    const empty = await server.fetch(new Request("http://localhost:4000/todos"));
    const emptyList = JSON.parse(await empty.text());
    results.push(["empty-list", Array.isArray(emptyList) && emptyList.length === 0]);

    const createReq = new Request("http://localhost:4000/todos", {
        method: "POST",
        body: JSON.stringify({ title: "buy milk" }),
    });
    const created = await server.fetch(createReq);
    const createdBody = JSON.parse(await created.text());
    results.push(["create", created.status === 201 && createdBody.title === "buy milk" && !createdBody.done]);

    const listedRes = await server.fetch(new Request("http://localhost:4000/todos"));
    const listed = JSON.parse(await listedRes.text());
    results.push(["list-has-one", listed.length === 1]);

    const doneRes = await server.fetch(new Request("http://localhost:4000/todos/" + createdBody.id + "/done", { method: "POST" }));
    const doneBody = JSON.parse(await doneRes.text());
    results.push(["mark-done", doneRes.status === 200 && doneBody.done === true]);

    const ftRes = await server.fetch(new Request("http://localhost:4000/todos?done=true"));
    const filteredTrue = JSON.parse(await ftRes.text());
    results.push(["filter-done-true", filteredTrue.length === 1 && filteredTrue[0].done]);

    const ffRes = await server.fetch(new Request("http://localhost:4000/todos?done=false"));
    const filteredFalse = JSON.parse(await ffRes.text());
    results.push(["filter-done-false", filteredFalse.length === 0]);

    const notFound = await server.fetch(new Request("http://localhost:4000/nonexistent"));
    results.push(["catch-all-404", notFound.status === 404]);

    const badReq = new Request("http://localhost:4000/todos", { method: "POST", body: JSON.stringify({}) });
    const bad = await server.fetch(badReq);
    results.push(["validate-400", bad.status === 400]);

    results.push(["count-final", listed.length === 1]);

    return results;
}

const results = await runSelfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    process.stdout.write(summary + "\n");
} else {
    globalThis.__esmResult = summary;
}
