// Tier-J consumer #40: vendored csv-mini RFC 4180 parser+writer.
//
// Exercises state-machine string processing — different axis from
// the regex-based mustache parser (csv-mini uses no regex in the
// hot path; pure char-by-char state machine, O(n)). Also exercises
// the default-export + named-exports pattern + RegExp construction
// from string + spread + array map.

import csv, { parse, stringify, parseObjects } from "csv-mini";

async function selfTest() {
    const results = [];

    // 1. Trivial 2-row CSV with LF endings.
    {
        const rows = parse("a,b,c\n1,2,3");
        results.push(["lf-basic",
            JSON.stringify(rows) === JSON.stringify([["a","b","c"], ["1","2","3"]])]);
    }

    // 2. CRLF endings (RFC 4180 canonical).
    {
        const rows = parse("a,b\r\n1,2\r\n3,4");
        results.push(["crlf-basic",
            JSON.stringify(rows) === JSON.stringify([["a","b"], ["1","2"], ["3","4"]])]);
    }

    // 3. Quoted field with embedded comma.
    {
        const rows = parse('name,greeting\nalice,"hello, world"');
        results.push(["quoted-embedded-comma",
            JSON.stringify(rows) === JSON.stringify([["name","greeting"], ["alice","hello, world"]])]);
    }

    // 4. Quoted field with embedded quote (RFC 4180 "" escape).
    {
        const rows = parse('a,b\n"she said ""hi""",end');
        results.push(["quoted-embedded-quote",
            JSON.stringify(rows) === JSON.stringify([["a","b"], ['she said "hi"',"end"]])]);
    }

    // 5. Quoted field with embedded newline.
    {
        const rows = parse('a,b\n"line1\nline2",end');
        results.push(["quoted-embedded-newline",
            JSON.stringify(rows) === JSON.stringify([["a","b"], ["line1\nline2","end"]])]);
    }

    // 6. Empty fields.
    {
        const rows = parse("a,,c\n,2,");
        results.push(["empty-fields",
            JSON.stringify(rows) === JSON.stringify([["a","","c"], ["","2",""]])]);
    }

    // 7. Trailing newline doesn't produce a phantom empty row.
    {
        const rows = parse("a,b\n1,2\n");
        results.push(["trailing-newline",
            rows.length === 2]);
    }

    // 8. stringify round-trip preserves data through quoting.
    {
        const original = [["name","note"], ["alice",'hello, "world"'], ["bob","line1\nline2"]];
        const text = stringify(original);
        const back = parse(text);
        results.push(["stringify-roundtrip",
            JSON.stringify(back) === JSON.stringify(original)]);
    }

    // 9. stringify quotes only when needed.
    {
        const text = stringify([["plain","field"], ["needs,quoting","ok"]]);
        results.push(["stringify-minimal-quoting",
            text === 'plain,field\r\n"needs,quoting",ok']);
    }

    // 10. parseObjects: header-mode adapter common in production code.
    {
        const objs = parseObjects("name,age\nalice,30\nbob,25");
        results.push(["parse-objects",
            objs.length === 2 &&
            objs[0].name === "alice" && objs[0].age === "30" &&
            objs[1].name === "bob"   && objs[1].age === "25"]);
    }

    // 11. Default-export shape (commonjs-compat).
    {
        const rows = csv.parse("a\nb");
        results.push(["default-export",
            typeof csv.parse === "function" &&
            typeof csv.stringify === "function" &&
            JSON.stringify(rows) === JSON.stringify([["a"], ["b"]])]);
    }

    // 12. Malformed (unterminated quote) throws.
    {
        let threw = false;
        try { parse('"open without close'); } catch (_) { threw = true; }
        results.push(["unterminated-quote-throws", threw]);
    }

    // 13. Alternate delimiter (tab) — TSV.
    {
        const rows = parse("a\tb\tc\n1\t2\t3", { delimiter: "\t" });
        results.push(["alternate-delimiter",
            JSON.stringify(rows) === JSON.stringify([["a","b","c"], ["1","2","3"]])]);
    }

    // 14. Real-world: parse a CSV with mixed quoting then stringify, expect
    //     output round-trips back through parse to the same logical data.
    {
        const input = 'id,name,note\r\n1,alice,"first, with comma"\r\n2,bob,"escaped ""quote"""\r\n3,carol,plain';
        const rows = parse(input);
        const out = stringify(rows);
        const back = parse(out);
        results.push(["real-world-roundtrip",
            JSON.stringify(back) === JSON.stringify(rows)]);
    }

    // 15. Performance-shaped: many rows.
    {
        const rows = [];
        for (let i = 0; i < 1000; i++) rows.push([String(i), "row-" + i, "x".repeat(20)]);
        const text = stringify(rows);
        const back = parse(text);
        results.push(["many-rows",
            back.length === 1000 && back[999][1] === "row-999"]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
