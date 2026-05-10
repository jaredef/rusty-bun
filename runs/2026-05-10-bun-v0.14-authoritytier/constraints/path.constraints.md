# path — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: path-surface-property
  threshold: PATH1
  interface: [path.win32.isAbsolute, path.toNamespacedPath, path.parse, path.posix.isAbsolute, path.posix.toNamespacedPath, path]

@imports: []

@pins: []

Surface drawn from 21 candidate properties across the Bun test corpus. Construction-style: 6; behavioral (high-cardinality): 15. Total witnessing constraint clauses: 375.

## PATH1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.isAbsolute** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 22 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:30` — path > zero length strings → `assert.strictEqual(path.win32.isAbsolute(""), false)`
- `test/js/node/path/is-absolute.test.js:7` — path.isAbsolute > win32 → `assert.strictEqual(path.win32.isAbsolute("/"), true)`
- `test/js/node/path/browserify.test.js:917` — browserify path tests > isAbsolute > win32 /foo/bar → `expect(path.win32.isAbsolute("/foo/bar")).toBe(true)`

## PATH2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.toNamespacedPath** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/to-namespaced-path.test.js:11` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(""), "")`
- `test/js/node/path/to-namespaced-path.test.js:12` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(null), null)`
- `test/js/node/path/to-namespaced-path.test.js:13` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(100), 100)`

## PATH3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/parse-format.test.js:168` — path.parse > general → `assert.strictEqual(typeof output.root, "string")`
- `test/js/node/path/parse-format.test.js:169` — path.parse > general → `assert.strictEqual(typeof output.dir, "string")`
- `test/js/node/path/parse-format.test.js:170` — path.parse > general → `assert.strictEqual(typeof output.base, "string")`

## PATH4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.isAbsolute** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:29` — path > zero length strings → `assert.strictEqual(path.posix.isAbsolute(""), false)`
- `test/js/node/path/is-absolute.test.js:28` — path.isAbsolute > posix → `assert.strictEqual(path.posix.isAbsolute("/home/foo"), true)`
- `test/js/node/path/browserify.test.js:918` — browserify path tests > isAbsolute > posix /foo/bar → `expect(path.posix.isAbsolute("/foo/bar")).toBe(true)`

## PATH5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.toNamespacedPath** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/to-namespaced-path.test.js:74` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath("/foo/bar"), "/foo/bar")`
- `test/js/node/path/to-namespaced-path.test.js:75` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath("foo/bar"), "foo/bar")`
- `test/js/node/path/to-namespaced-path.test.js:76` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath(null), null)`

## PATH6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/path/path.test.js:58` — path > path.delimiter → `assert.strictEqual(path, path.win32)`
- `test/js/node/fs/glob.test.ts:206` — fs.promises.glob > matches directories → `expect(path).toBe("folder.test")`
- `test/js/bun/resolve/load-same-js-file-a-lot.test.ts:27` — load the same file ${count} times → `expect(path).toBe(meta.path)`

## PATH7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.dirname** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 98)

Witnessed by 98 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/dirname.test.js:12` — path.dirname > win32 → `assert.strictEqual(path.win32.dirname("c:\\"), "c:\\")`
- `test/js/node/path/browserify.test.js:82` — browserify path tests > dirname > path.win32.dirname → `expect(path.win32.dirname("c:\\")).toBe("c:\\")`
- `test/js/node/path/dirname.test.js:13` — path.dirname > win32 → `assert.strictEqual(path.win32.dirname("c:\\foo"), "c:\\")`

## PATH8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.dirname** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 47)

Witnessed by 47 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/dirname.test.js:51` — path.dirname > posix → `assert.strictEqual(path.posix.dirname("/a/b/"), "/a")`
- `test/js/node/path/browserify.test.js:35` — browserify path tests > dirname > path.dirname → `expect(path.posix.dirname(input)).toBe(expected)`
- `test/js/node/path/dirname.test.js:52` — path.dirname > posix → `assert.strictEqual(path.posix.dirname("/a/b"), "/a")`

## PATH9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.basename** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 31)

Witnessed by 31 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/basename.test.js:7` — path.dirname > platform → `assert.strictEqual(path.basename(__filename), "basename.test.js")`
- `test/js/node/path/basename.test.js:8` — path.dirname > platform → `assert.strictEqual(path.basename(__filename, ".js"), "basename.test")`
- `test/js/node/path/basename.test.js:9` — path.dirname > platform → `assert.strictEqual(path.basename(".js", ".js"), "")`

## PATH10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.normalize** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 25)

Witnessed by 25 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:25` — path > zero length strings → `assert.strictEqual(path.win32.normalize(""), ".")`
- `test/js/node/path/normalize.test.js:7` — path.normalize > win32 → `assert.strictEqual(path.win32.normalize("./fixtures///b/../b/c.js"), "fixtures\\b\\c.js")`
- `test/js/node/path/normalize.test.js:8` — path.normalize > win32 → `assert.strictEqual(path.win32.normalize("/foo/../../../bar"), "\\bar")`

## PATH11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.basename** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 24)

Witnessed by 24 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/basename.test.js:42` — path.dirname > win32 → `assert.strictEqual(path.win32.basename("\\dir\\basename.ext"), "basename.ext")`
- `test/js/node/path/basename.test.js:43` — path.dirname > win32 → `assert.strictEqual(path.win32.basename("\\basename.ext"), "basename.ext")`
- `test/js/node/path/basename.test.js:44` — path.dirname > win32 → `assert.strictEqual(path.win32.basename("basename.ext"), "basename.ext")`

## PATH12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.normalize** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 20)

Witnessed by 20 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:24` — path > zero length strings → `assert.strictEqual(path.posix.normalize(""), ".")`
- `test/js/node/path/normalize.test.js:34` — path.normalize > posix → `assert.strictEqual(path.posix.normalize("./fixtures///b/../b/c.js"), "fixtures/b/c.js")`
- `test/js/node/path/normalize.test.js:35` — path.normalize > posix → `assert.strictEqual(path.posix.normalize("/foo/../../../bar"), "/bar")`

## PATH13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.toNamespacedPath** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/to-namespaced-path.test.js:37` — path.toNamespacedPath > platform → `assert.strictEqual( path.win32.toNamespacedPath("foo\\bar").toLowerCase(), '\\\\?\\${process.cwd().toLowerCase()}\\foo\\bar', )`
- `test/js/node/path/to-namespaced-path.test.js:41` — path.toNamespacedPath > platform → `assert.strictEqual( path.win32.toNamespacedPath("foo/bar").toLowerCase(), '\\\\?\\${process.cwd().toLowerCase()}\\foo\\bar', )`
- `test/js/node/path/to-namespaced-path.test.js:46` — path.toNamespacedPath > platform → `assert.strictEqual( path.win32.toNamespacedPath(currentDeviceLetter).toLowerCase(), '\\\\?\\${process.cwd().toLowerCase()}', )`

## PATH14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.basename** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/basename.test.js:70` — path.dirname > posix → `assert.strictEqual(path.posix.basename("\\dir\\basename.ext"), "\\dir\\basename.ext")`
- `test/js/node/path/basename.test.js:71` — path.dirname > posix → `assert.strictEqual(path.posix.basename("\\basename.ext"), "\\basename.ext")`
- `test/js/node/path/basename.test.js:72` — path.dirname > posix → `assert.strictEqual(path.posix.basename("basename.ext"), "basename.ext")`

## PATH15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.extname** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/extname.test.js:98` — path.extname > posix → `assert.strictEqual(path.posix.extname(".\\"), "")`
- `test/js/node/path/extname.test.js:99` — path.extname > posix → `assert.strictEqual(path.posix.extname("..\\"), ".\\")`
- `test/js/node/path/extname.test.js:100` — path.extname > posix → `assert.strictEqual(path.posix.extname("file.ext\\"), ".ext\\")`

## PATH16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.extname** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/extname.test.js:86` — path.extname > win32 → `assert.strictEqual(path.win32.extname(".\\"), "")`
- `test/js/node/path/extname.test.js:87` — path.extname > win32 → `assert.strictEqual(path.win32.extname("..\\"), "")`
- `test/js/node/path/extname.test.js:88` — path.extname > win32 → `assert.strictEqual(path.win32.extname("file.ext\\"), ".ext")`

## PATH17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.relative** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:38` — path > zero length strings → `assert.strictEqual(path.relative("", pwd), "")`
- `test/bundler/bun-build-api.test.ts:221` — Bun.build > BuildArtifact properties → `expect(path.relative(outdir, blob.path)).toBe("index.js")`
- `test/js/node/path/zero-length-strings.test.js:39` — path > zero length strings → `assert.strictEqual(path.relative(pwd, ""), "")`

## PATH18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.format** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/path/parse-format.test.js:173` — path.parse > general → `assert.strictEqual(path.format(output), element)`
- `test/js/node/path/browserify.test.js:900` — browserify path tests > path.format works for vite's example → `expect( path.format({ root: "", dir: "", name: "index", base: undefined, ext: ".css", }), ).toBe("index.css")`
- `test/js/node/path/parse-format.test.js:190` — path.parse > general → `assert.strictEqual(path.format(element), expect)`

## PATH19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.join** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:20` — path > zero length strings → `assert.strictEqual(path.join(pwd), pwd)`
- `test/js/node/path/browserify.test.js:315` — browserify path tests > path.join #5769 → `expect(path.join(tooLengthyFolderName)).toEqual("b".repeat(length))`
- `test/js/node/path/15704.test.js:7` — too-long path names do not crash when joined → `assert.equal(path.join(tooLengthyFolderName), "b".repeat(length))`

## PATH20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.normalize** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:26` — path > zero length strings → `assert.strictEqual(path.normalize(pwd), pwd)`
- `test/js/node/path/normalize.test.js:61` — path.normalize > very long paths → `assert.strictEqual(path.normalize(longPath), longPath)`
- `test/js/bun/spawn/spawn.test.ts:1012` — argv0 > argv0 option changes process.argv0 but not executable → `expect(path.normalize(lines[1])).toBe(path.normalize(bunExe()))`

## PATH21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.resolve** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/resolve.test.js:105` — path.resolve > UNC path before drive-relative path does not corrupt resolvedDevice → `expect(path.win32.resolve("C:/base", "//server/share", "C:relative")).toBe("C:\\base\\relative")`
- `test/js/node/path/resolve.test.js:106` — path.resolve > UNC path before drive-relative path does not corrupt resolvedDevice → `expect(path.win32.resolve("C:/base", "//a/b", "//c/d", "C:foo")).toBe("C:\\base\\foo")`
- `test/js/node/path/resolve.test.js:109` — path.resolve > UNC path before drive-relative path does not corrupt resolvedDevice → `expect(path.win32.resolve("//server/share", "C:relative")).toBe(path.win32.resolve("C:relative"))`

