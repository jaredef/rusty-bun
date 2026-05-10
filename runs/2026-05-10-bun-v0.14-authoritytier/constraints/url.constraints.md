# url — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: url-surface-property
  threshold: URL1
  interface: [url.includes, url.href.endsWith, url.searchParams.get]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 77.

## URL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.includes** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 60 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:1306` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Expires=86400")).toBe(true)`
- `test/js/bun/s3/s3.test.ts:1307` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Date")).toBe(true)`
- `test/js/bun/s3/s3.test.ts:1308` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Signature")).toBe(true)`

## URL2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.href.endsWith** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/fileUrl.test.js:13` — pathToFileURL > should handle relative paths longer than PATH_MAX → `expect(url.href.endsWith("/" + long)).toBe(true)`

## URL3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**url.searchParams.get** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/s3/s3-requester-pays.test.ts:249` — s3 - Requester Pays > should NOT include x-amz-request-payer in presigned URLs when reques… → `expect(url.searchParams.get("x-amz-request-payer")).toBeNull()`

## URL4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.format** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/url/url-parse-format.test.js:1052` — url.parse then url.format > url.format → `assert.strictEqual(actual, expected, 'format(${u}) == ${u}\nactual:${actual}')`
- `test/js/node/url/url-format.test.js:273` — url.format > slightly wonky content → `assert.strictEqual(actual, expect, 'wonky format(${u}) == ${expect}\nactual:${actual}')`
- `test/js/node/url/url-format-whatwg.test.js:29` — url.format > WHATWG → `assert.strictEqual(url.format(myURL, { auth: false }), "http://xn--lck1c3crb1723bpq4a.com/a?a=b#c")`

## URL5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.domainToASCII** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/24191.test.ts:8` — url.domainToASCII returns empty string for invalid domains → `expect(url.domainToASCII("xn--iñvalid.com")).toBe("")`
- `test/js/node/url/url-domain-ascii-unicode.test.js:79` — url.domainToASCII > convert from '${domain}' to '${ascii}' → `expect(domainConvertedToASCII).toEqual(ascii)`
- `test/regression/issue/24191.test.ts:11` — url.domainToASCII returns empty string for invalid domains → `expect(url.domainToASCII("example.com")).toBe("example.com")`

