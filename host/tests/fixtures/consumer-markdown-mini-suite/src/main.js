// Tier-J consumer #41: vendored markdown-mini renderer.
//
// Different parser shape from prior fixtures: two-pass (block-level
// then inline) with multiple regex patterns interacting per pass and
// a sentinel-substitution scheme to avoid re-processing escaped or
// code-protected text. Closer to production markdown libraries than
// either mustache (single-pass interpolation) or csv (state machine).

import markdown, { render } from "markdown-mini";

async function selfTest() {
    const results = [];

    // 1. ATX heading.
    results.push(["h1",
        render("# Hello") === "<h1>Hello</h1>"]);

    // 2. Multi-level heading.
    results.push(["h3",
        render("### Section") === "<h3>Section</h3>"]);

    // 3. Paragraph.
    results.push(["paragraph",
        render("Just text.") === "<p>Just text.</p>"]);

    // 4. Inline bold.
    results.push(["bold",
        render("a **strong** word.") === "<p>a <strong>strong</strong> word.</p>"]);

    // 5. Inline italic.
    results.push(["italic",
        render("a *strong* word.") === "<p>a <em>strong</em> word.</p>"]);

    // 6. Inline code with HTML-special chars.
    results.push(["inline-code-escaped",
        render("Use `<script>`") === "<p>Use <code>&lt;script&gt;</code></p>"]);

    // 7. Link.
    results.push(["link",
        render("See [docs](https://example.com).") ===
        '<p>See <a href="https://example.com">docs</a>.</p>']);

    // 8. Fenced code block with language.
    results.push(["fenced-code-with-lang",
        render("```js\nconst x = 1;\n```") ===
        '<pre><code class="language-js">const x = 1;</code></pre>']);

    // 9. Fenced code preserves angle brackets.
    results.push(["fenced-code-escapes-html",
        render("```\n<b>not bold</b>\n```") ===
        '<pre><code>&lt;b&gt;not bold&lt;/b&gt;</code></pre>']);

    // 10. Blockquote.
    results.push(["blockquote",
        render("> quoted text") === "<blockquote><p>quoted text</p></blockquote>"]);

    // 11. Unordered list.
    results.push(["list",
        render("- one\n- two\n- three") ===
        "<ul><li>one</li><li>two</li><li>three</li></ul>"]);

    // 12. Multiple blocks separated by blank lines.
    results.push(["multi-block",
        render("# Title\n\nFirst paragraph.\n\nSecond paragraph.") ===
        "<h1>Title</h1><p>First paragraph.</p><p>Second paragraph.</p>"]);

    // 13. HTML in paragraph is escaped (security-critical).
    results.push(["paragraph-escapes-html",
        render("alert: <script>x</script>") ===
        "<p>alert: &lt;script&gt;x&lt;/script&gt;</p>"]);

    // 14. Backslash escape: \* should render literal *.
    results.push(["backslash-escape-star",
        render("not \\*emphasized\\*") === "<p>not *emphasized*</p>"]);

    // 15. Mixed inline: bold inside paragraph with code.
    results.push(["mixed-inline",
        render("a **bold** with `code` and *em*.") ===
        "<p>a <strong>bold</strong> with <code>code</code> and <em>em</em>.</p>"]);

    // 16. Link URL with query string is escaped.
    results.push(["link-url-escape",
        render("[q](https://a.example/?x=1&y=2)") ===
        '<p><a href="https://a.example/?x=1&amp;y=2">q</a></p>']);

    // 17. Nested blockquote.
    results.push(["nested-blockquote",
        render("> outer\n> > inner") ===
        "<blockquote><p>outer</p><blockquote><p>inner</p></blockquote></blockquote>"]);

    // 18. Default-export shape.
    results.push(["default-export",
        typeof markdown.render === "function" &&
        markdown.render("# X") === "<h1>X</h1>"]);

    // 19. Bold inside heading.
    results.push(["heading-with-bold",
        render("# Title with **bold**") === "<h1>Title with <strong>bold</strong></h1>"]);

    // 20. Real-world short doc.
    const doc = `# Project README

This project demonstrates **markdown rendering**.

## Features

- Heading levels
- *Italic* and **bold** text
- Inline \`code\`
- [Documentation links](https://docs.example.com)

## Code Example

\`\`\`js
const greet = (name) => \`Hello, \${name}!\`;
\`\`\`

> Note: this is a tiny subset.`;
    const out = render(doc);
    results.push(["real-world-doc",
        out.startsWith("<h1>Project README</h1>") &&
        out.includes("<strong>bold</strong>") &&
        out.includes('<a href="https://docs.example.com">Documentation links</a>') &&
        out.includes('<pre><code class="language-js">') &&
        out.includes("<blockquote>") &&
        out.includes("<ul>") &&
        out.includes("<h2>Features</h2>")]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
