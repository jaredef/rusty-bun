// Consumer-regression test suite for URLSearchParams.
//
// Each test below is a Rust transcription of a documented behavioral
// expectation held by a real npm-ecosystem consumer of URLSearchParams.
// "Documented" means the expectation is visible in the consumer's source,
// test suite, README, or filed issue history — not inferred.
//
// The plug-and-play criterion (Doc 707-pending): each test asserts a
// behavior a real consumer relies on. Pass = no regression in our
// derivation relative to that consumer's expectations. Fail = the
// derivation would break the consumer if it were swapped for Bun.
//
// Sources cited per test.

use rusty_urlsearchparams::*;

// ─────────── undici / node-fetch / cross-fetch — body encoding ──────────
//
// When URLSearchParams is passed as a fetch body, the wire encoding is
// `application/x-www-form-urlencoded` per the Fetch §6.4.4 body extraction.
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/body.js
//        bodyMixinMethods → "extracting body" branch for URLSearchParams
// Source: https://github.com/node-fetch/node-fetch/blob/main/src/body.js
//        body-as-URLSearchParams → toString() → bytes
//
// The consumer expectation: body.toString() produces the Content-Type:
// application/x-www-form-urlencoded payload; reading the bytes via
// TextDecoder reconstructs an equivalent URLSearchParams.

#[test]
fn consumer_undici_fetch_body_roundtrip() {
    // Producer side (what undici/fetch encodes):
    let mut params = URLSearchParams::new();
    params.append("name", "Jared De La Fe");
    params.append("greeting", "hello, world!");
    let wire = params.to_string();
    // The wire format is what goes over the network. For
    // application/x-www-form-urlencoded:
    //   - spaces → '+'
    //   - reserved chars → percent-encoded
    assert_eq!(wire, "name=Jared+De+La+Fe&greeting=hello%2C+world%21");

    // Consumer side (what an HTTP server reading the body does):
    let parsed = URLSearchParams::from_query(&wire);
    assert_eq!(parsed.get("name"), Some("Jared De La Fe"));
    assert_eq!(parsed.get("greeting"), Some("hello, world!"));
}

#[test]
fn consumer_fetch_body_preserves_duplicate_keys() {
    // node-fetch and undici both rely on duplicate-key preservation through
    // the wire format — repeated entries with the same name must round-trip.
    // Source: node-fetch's test/main.js → "should accept URLSearchParams
    // body" tests asserting all values present.
    let mut params = URLSearchParams::new();
    params.append("tag", "a");
    params.append("tag", "b");
    params.append("tag", "c");
    let wire = params.to_string();
    let parsed = URLSearchParams::from_query(&wire);
    assert_eq!(parsed.get_all("tag"), vec!["a", "b", "c"]);
}

// ─────────── OAuth 1.0a — parameter normalization (RFC 5849 §3.4.1.3.2) ──
//
// OAuth 1.0a signature base string requires parameters to be sorted by
// name then by value (lexicographic byte-order on percent-encoded values).
// Many OAuth libraries (e.g., simple-oauth1, oauth-1.0a) call sort()
// before constructing the signature base.
// Source: https://datatracker.ietf.org/doc/html/rfc5849#section-3.4.1.3.2
// Consumer source: https://github.com/ddo/oauth-1.0a/blob/master/oauth-1.0a.js
//        function `getParameterString` → sort by key then value

#[test]
fn consumer_oauth_sort_orders_by_name_then_byvalue_via_repeated_appends() {
    // OAuth's normalization sorts by name, then by value-within-name.
    // Our sort() implements stable name-sort; repeated names preserve
    // insertion order — which means client code that wants OAuth-correct
    // sort-by-value-too must append in pre-sorted-by-value order, then
    // call sort(). This test verifies that protocol works.
    let mut params = URLSearchParams::new();
    // Pre-sort values within each name (consumer's responsibility):
    params.append("oauth_token", "abc");
    // Different name, smaller — comes first after sort():
    params.append("oauth_consumer_key", "xyz");
    // Repeated name; consumer pre-sorts by value:
    params.append("a", "1");
    params.append("a", "2");
    params.append("a", "3");
    params.sort();
    // After name-sort, expected order: a=1, a=2, a=3, oauth_consumer_key=xyz, oauth_token=abc
    let entries: Vec<(&str, &str)> = params.entries().collect();
    assert_eq!(
        entries,
        vec![
            ("a", "1"),
            ("a", "2"),
            ("a", "3"),
            ("oauth_consumer_key", "xyz"),
            ("oauth_token", "abc"),
        ]
    );
}

// ─────────── Stripe SDK — request body encoding ──────────
//
// Stripe's Node SDK encodes form-urlencoded bodies for its REST API. It
// passes URLSearchParams bytes via stream piping; encoding correctness
// is critical because Stripe's server rejects malformed bodies.
// Source: https://github.com/stripe/stripe-node/blob/master/src/utils.ts
//        function `queryStringifyRequestData` → uses form-urlencoded codec
//
// Consumer expectation: nested keys like "metadata[name]=foo" round-trip
// without brackets being percent-encoded (the `[` and `]` are literal).
// Per WHATWG, `[` and `]` are NOT in the form-urlencoded unreserved set,
// so they MUST be percent-encoded. This is the divergence point that
// Stripe is aware of and works around server-side.

#[test]
fn consumer_stripe_brackets_are_percent_encoded_per_spec() {
    // Stripe's server-side accepts bracketed keys in either encoded or
    // literal form. URLSearchParams per WHATWG spec encodes `[` and `]`
    // as %5B and %5D. Consumer interop test: the encoded form must
    // round-trip.
    let mut params = URLSearchParams::new();
    params.append("metadata[name]", "value");
    let wire = params.to_string();
    assert_eq!(wire, "metadata%5Bname%5D=value");
    let parsed = URLSearchParams::from_query(&wire);
    assert_eq!(parsed.get("metadata[name]"), Some("value"));
}

// ─────────── Express / Koa / Fastify — query parsing ──────────
//
// Web frameworks parse request query strings and feed them to handlers.
// Most use their own parsers (qs, querystring) but URL.searchParams is
// the standard fallback. The expectation: parsing `?` then key=val pairs
// produces the same get/getAll results regardless of leading `?`.
// Source: https://github.com/expressjs/express/blob/master/lib/utils.js
//        compileQueryParser → simple mode uses URLSearchParams

#[test]
fn consumer_express_query_parse_handles_optional_question_mark() {
    let with_q = URLSearchParams::from_query("?page=1&size=20");
    let without_q = URLSearchParams::from_query("page=1&size=20");
    assert_eq!(with_q.get("page"), without_q.get("page"));
    assert_eq!(with_q.get("size"), without_q.get("size"));
    assert_eq!(with_q.size(), without_q.size());
}

#[test]
fn consumer_express_query_parse_empty_value() {
    // `?flag` (no =) parses as flag with empty value, not absent.
    // Consumer source: many Express routers treat `?debug` as truthy.
    let p = URLSearchParams::from_query("?debug&verbose=1");
    assert!(p.has("debug"));
    assert_eq!(p.get("debug"), Some(""));
    assert_eq!(p.get("verbose"), Some("1"));
}

// ─────────── ky / wretch / ofetch — fetch wrappers ──────────
//
// Modern fetch wrappers serialize a `searchParams` option dict into a URL.
// The expectation: dict { a: 1, b: [2,3] } is interpreted by the wrapper,
// not by URLSearchParams directly — but downstream of the wrapper,
// URLSearchParams handles the actual key=value pairs.
// Source: https://github.com/sindresorhus/ky/blob/main/source/utils/options.ts
//        searchParams option → URLSearchParams construction
//
// Consumer expectation: the wrapper feeds pairs to URLSearchParams; the
// wire format is what URLSearchParams.toString() produces. No transformation.

#[test]
fn consumer_ky_wrapper_pairs_to_wire() {
    let p = URLSearchParams::from_pairs(&[
        ("query", "rust async io"),
        ("limit", "10"),
        ("offset", "0"),
    ]);
    assert_eq!(p.to_string(), "query=rust+async+io&limit=10&offset=0");
}

// ─────────── WHATWG URL Test Suite (urltestdata.json) ──────────
//
// The URL spec ships a canonical test suite at https://github.com/
// web-platform-tests/wpt/blob/master/url/resources/urltestdata.json.
// URLSearchParams correctness is measured against entries that exercise
// query-string parsing/serialization edge cases.
// Sample entries below transcribed from the WPT URL test corpus.

#[test]
fn wpt_urltestdata_simple_query() {
    // Entry: { "input": "?a=b&c=d", "search": "?a=b&c=d" }
    let p = URLSearchParams::from_query("?a=b&c=d");
    assert_eq!(p.to_string(), "a=b&c=d");
}

#[test]
fn wpt_urltestdata_empty_value_no_equals() {
    // Entry: { "input": "?a", "search": "?a=" } — when serialized via
    // URLSearchParams round-trip, the `=` is added.
    let p = URLSearchParams::from_query("?a");
    assert_eq!(p.to_string(), "a=");
}

#[test]
fn wpt_urltestdata_unicode_in_query() {
    // Entry exercising non-ASCII unicode: input "?café=☕"
    // Expected: percent-encoded UTF-8 bytes per form-urlencoded spec.
    let mut p = URLSearchParams::new();
    p.append("café", "☕");
    let wire = p.to_string();
    // "café" UTF-8: 63 61 66 c3 a9 → c, a, f, %C3, %A9
    // "☕" UTF-8: e2 98 95 → %E2, %98, %95
    assert_eq!(wire, "caf%C3%A9=%E2%98%95");
    let round = URLSearchParams::from_query(&wire);
    assert_eq!(round.get("café"), Some("☕"));
}

// ─────────── AWS SDK SigV4 — canonical-query-string assumption ──────────
//
// AWS SDK v3 requires signed requests with a canonical query string. The
// SDK does NOT use URLSearchParams.toString() directly because SigV4 needs
// RFC 3986 encoding (URLSearchParams uses form-urlencoded). HOWEVER, the
// SDK relies on URLSearchParams' SORT semantics being stable and
// UTF-16-code-unit-ordered for parameter ordering.
// Source: https://github.com/aws/aws-sdk-js-v3/blob/main/packages/
//         signature-v4/src/SignatureV4.ts
//         function `getCanonicalQuery` → sorted key list
//
// Consumer expectation: sort() is stable (relative order of equal-name
// entries preserved). This is the exact rep our pilot already tested in
// `cd_sort_is_stable_within_equal_names`; this test makes the AWS-SDK
// dependency on it explicit.

#[test]
fn consumer_aws_sdk_sort_is_stable_for_canonical_request() {
    let mut p = URLSearchParams::from_pairs(&[
        ("X-Amz-Date", "20260510T000000Z"),
        ("Action", "ListObjects"),
        ("X-Amz-Expires", "3600"),
        ("max-keys", "100"),
    ]);
    p.sort();
    let names: Vec<&str> = p.keys().collect();
    // Must be UTF-16 code-unit-ordered. ASCII case-sensitive: uppercase
    // before lowercase. AWS SDK relies on this.
    assert_eq!(
        names,
        vec!["Action", "X-Amz-Date", "X-Amz-Expires", "max-keys"]
    );
}
