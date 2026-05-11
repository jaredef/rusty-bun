// Tier-J consumer #39: vendored mustache-mini template engine.
//
// Exercises a fundamentally non-crypto axis: RegExp-heavy template
// parsing + recursive AST walk + HTML escape + Map-based template
// cache + dotted-name context lookup + section iteration. Pure JS,
// no host bindings beyond what the apparatus already wires.

import mustache, { compile, renderTemplate } from "mustache-mini";

async function selfTest() {
    const results = [];

    // 1. Simple interpolation with HTML escape.
    {
        const out = renderTemplate(
            "Hello, {{name}}!",
            { name: "<script>" });
        results.push(["escape-default",
            out === "Hello, &lt;script&gt;!"]);
    }

    // 2. Triple-stache for unescaped.
    {
        const out = renderTemplate(
            "Body: {{{html}}}",
            { html: "<b>bold</b>" });
        results.push(["raw-triple-stache",
            out === "Body: <b>bold</b>"]);
    }

    // 3. Ampersand alias for unescaped.
    {
        const out = renderTemplate(
            "URL: {{&url}}",
            { url: "https://x.example/?q=1&p=2" });
        results.push(["raw-ampersand",
            out === "URL: https://x.example/?q=1&p=2"]);
    }

    // 4. Dotted-name context lookup.
    {
        const out = renderTemplate(
            "{{user.profile.email}}",
            { user: { profile: { email: "alice@example.com" } } });
        results.push(["dotted-name", out === "alice@example.com"]);
    }

    // 5. Section iteration over array of objects.
    {
        const out = renderTemplate(
            "Items:{{#items}} - {{title}}({{price}}){{/items}}",
            { items: [{ title: "A", price: 1 }, { title: "B", price: 2 }] });
        results.push(["section-iteration",
            out === "Items: - A(1) - B(2)"]);
    }

    // 6. Section on a falsy / empty array skips.
    {
        const out = renderTemplate(
            "[{{#items}}{{.}},{{/items}}]",
            { items: [] });
        results.push(["section-empty-array-skipped", out === "[]"]);
    }

    // 7. Inverted section fires on falsy.
    {
        const out = renderTemplate(
            "{{^empty}}has items{{/empty}}{{#empty}}no items{{/empty}}",
            { empty: false });
        results.push(["inverted-section", out === "has items"]);
    }

    // 8. Comments are stripped.
    {
        const out = renderTemplate(
            "before{{! a comment }}after", {});
        results.push(["comment-stripped", out === "beforeafter"]);
    }

    // 9. Iteration over primitive array via {{.}}.
    {
        const out = renderTemplate(
            "[{{#nums}}{{.}},{{/nums}}]",
            { nums: [1, 2, 3] });
        results.push(["dot-current-context", out === "[1,2,3,]"]);
    }

    // 10. Nested section with context-stack walking.
    {
        const out = renderTemplate(
            "{{#users}}{{name}}-{{#roles}}{{.}}|{{/roles}};{{/users}}",
            { users: [
                { name: "alice", roles: ["admin", "editor"] },
                { name: "bob",   roles: ["viewer"] },
            ]});
        results.push(["nested-sections",
            out === "alice-admin|editor|;bob-viewer|;"]);
    }

    // 11. compile() returns a function that caches across calls.
    {
        const tpl = "Hello {{name}}";
        const a = compile(tpl);
        const b = compile(tpl);
        results.push(["compile-cache", a === b && a({ name: "X" }) === "Hello X"]);
    }

    // 12. Default export shape (object with compile + render methods).
    {
        const out = mustache.render("{{x}}", { x: "ok" });
        results.push(["default-export", out === "ok" && typeof mustache.compile === "function"]);
    }

    // 13. Inverted section on truthy is empty.
    {
        const out = renderTemplate(
            "{{^empty}}A{{/empty}}-{{^nonEmpty}}B{{/nonEmpty}}",
            { empty: false, nonEmpty: ["x"] });
        results.push(["inverted-truthy-empty", out === "A-"]);
    }

    // 14. Walking the context stack up (inner section accesses outer field).
    {
        const out = renderTemplate(
            "{{#users}}{{name}}@{{org}}|{{/users}}",
            { org: "acme", users: [{ name: "alice" }, { name: "bob" }] });
        results.push(["context-stack-walk", out === "alice@acme|bob@acme|"]);
    }

    // 15. Missing keys render as empty (mustache convention).
    {
        const out = renderTemplate(
            "[{{missing}}]", {});
        results.push(["missing-empty", out === "[]"]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
