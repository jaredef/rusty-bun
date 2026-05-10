# fs — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fs-surface-property
  threshold: FS1
  interface: [fs.existsSync, fs.Dirent, fs.globSync, fs.lstatSync, fs.promises.glob, fs.access, fs.promises.statfs, fs.ReadStream, fs.ReadStream, fs.WriteStream, fs.WriteStream, fs.glob, fs.mkdirSync, fs.promises.glob, fs.promises.mkdir]

@imports: []

@pins: []

Surface drawn from 24 candidate properties across the Bun test corpus. Construction-style: 15; behavioral (high-cardinality): 9. Total witnessing constraint clauses: 282.

## FS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.existsSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 76 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/13316.test.ts:44` — bunx does not panic with empty string argument → `expect(fs.existsSync(bunxPath)).toBe(true)`
- `test/js/node/fs/fs-mkdir.test.ts:52` — fs.mkdir > creates directory using assigned path → `expect(fs.existsSync(pathname)).toBe(true)`
- `test/js/node/fs/cp.test.ts:275` — filter - works → `expect(fs.existsSync(basename + "/result/a.txt")).toBe(true)`

## FS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.Dirent** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24129.test.ts:17` — Dirent with unknown type should return false for all type checks → `expect(dirent.isFile()).toBe(false)`
- `test/regression/issue/24129.test.ts:18` — Dirent with unknown type should return false for all type checks → `expect(dirent.isDirectory()).toBe(false)`
- `test/regression/issue/24129.test.ts:19` — Dirent with unknown type should return false for all type checks → `expect(dirent.isSymbolicLink()).toBe(false)`

## FS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.globSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/24007.test.ts:39` — issue #24007 - glob with recursive patterns > recursive glob pattern **/*.ts finds nested … → `expect(results.length).toBe(5)`
- `test/js/node/fs/glob.test.ts:91` — fs.glob > supports arrays of patterns → `expect(fs.globSync(["a/bar.txt", "a/baz.js"], { cwd: tmp })).toStrictEqual(expected)`
- `test/regression/issue/24007.test.ts:55` — issue #24007 - glob with recursive patterns > recursive glob pattern server/**/*.ts finds … → `expect(results.length).toBe(2)`

## FS4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.lstatSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/cp.test.ts:156` — symlinks - single file → `expect(stats.isSymbolicLink()).toBe(true)`
- `test/js/node/fs/cp.test.ts:160` — symlinks - single file → `expect(stats2.isSymbolicLink()).toBe(true)`
- `test/js/node/fs/cp.test.ts:174` — symlinks - single file recursive → `expect(stats.isSymbolicLink()).toBe(true)`

## FS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.glob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:176` — fs.promises.glob > returns an AsyncIterable over matched paths → `expect(iter[Symbol.asyncIterator]).toBeDefined()`
- `test/js/node/fs/glob.test.ts:188` — fs.promises.glob > works without providing options → `expect(iter[Symbol.asyncIterator]).toBeDefined()`
- `test/js/node/fs/glob.test.ts:203` — fs.promises.glob > matches directories → `expect(iter[Symbol.asyncIterator]).toBeDefined()`

## FS6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.access** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/bun-object/write.spec.ts:131` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(rootdir, constants.F_OK)).toBeFalsy()`
- `test/js/bun/bun-object/write.spec.ts:132` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(path.dirname(filepath), constants.F_OK)).toBeFalsy()`

## FS7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.statfs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3660` — fs.promises.statfs should work → `expect(stats).toBeDefined()`
- `test/js/node/fs/fs.test.ts:3665` — fs.promises.statfs should work with bigint → `expect(stats).toBeDefined()`

## FS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.ReadStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:2263` — fs.ReadStream > should be constructable → `expect(stream instanceof fs.ReadStream).toBe(true)`

## FS9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.ReadStream** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:2257` — fs.ReadStream > should be exported → `expect(fs.ReadStream).toBeDefined()`

## FS10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.WriteStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:2171` — fs.WriteStream > should be constructable → `expect(stream instanceof fs.WriteStream).toBe(true)`

## FS11
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.WriteStream** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:2165` — fs.WriteStream > should be exported → `expect(fs.WriteStream).toBeDefined()`

## FS12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:33` — fs.glob > has a length of 3 → `expect(typeof fs.glob).toEqual("function")`

## FS13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.mkdirSync** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs-mkdir.test.ts:291` — fs.mkdir - return values > mkdirSync returns undefined with recursive when no new folders … → `expect(result).toBeUndefined()`

## FS14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:165` — fs.promises.glob > has a length of 2 → `expect(typeof fs.promises.glob).toBe("function")`

## FS15
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.mkdir** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs-mkdir.test.ts:347` — fs.promises.mkdir > returns undefined with recursive when no new folders are created → `expect(result).toBeUndefined()`

## FS16
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fs.readFileSync** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 85)

Witnessed by 85 constraint clauses across 5 test files. Antichain representatives:

- `test/cli/install/migration/yarn-lock-migration.test.ts:51` — yarn.lock migration basic > simple yarn.lock migration produces correct bun.lock → `expect(bunLockContent).toMatchSnapshot("simple-yarn-migration")`
- `test/cli/install/migration/pnpm-migration-complete.test.ts:89` — PNPM Migration Complete Test Suite > comprehensive PNPM migration with all edge cases → `expect(basicLockfile).toContain('"lodash": "^4.17.21"')`
- `test/cli/install/migration/pnpm-lock-migration.test.ts:81` — pnpm-lock.yaml migration > simple pnpm lockfile migration produces correct bun.lock → `expect(bunLockContent).toMatchSnapshot("simple-pnpm-migration")`

## FS17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.statSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:1543` — writeFileSync > write file with mode, issue #3740 → `expect(stat.mode).toBe(isWindows ? 33206 : 33188)`
- `test/js/node/fs/fs-mkdir.test.ts:144` — fs.mkdir - recursive > creates nested directories when both top-level and sub-folders don'… → `expect(fs.statSync(pathname).isDirectory()).toBe(true)`
- `test/js/node/fs/fs.test.ts:3036` — utimesSync > works → `expect(newStats.mtime).toEqual(newModifiedTime)`

## FS18
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fs.globSync** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/24007.test.ts:34` — issue #24007 - glob with recursive patterns > recursive glob pattern **/*.ts finds nested … → `expect(results).toContain("config.ts")`
- `test/js/node/fs/glob.test.ts:86` — fs.glob > matches directories → `expect(paths).toContain("folder.test")`
- `test/cli/run/glob-on-fuse.test.ts:72` — fs.globSync finds files on FUSE mount → `expect(results).toContain("main.js")`

## FS19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.readFileSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/utf16-encoding-crash.test.ts:50` — fs.readFile with utf16le encoding matches Node.js behavior for all byte lengths → `expect(result.length).toBe(testCase.expectedLength)`
- `test/js/node/fs/fs.test.ts:938` — promises.readFile > & fs.promises.writefile encodes & decodes → `expect(fs.readFileSync(outfile, encoding)).toEqual(out)`
- `test/js/node/fs/cp.test.ts:36` — single file → `expect(fs.readFileSync(basename + "/to.txt", "utf8")).toBe("a")`

## FS20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.readFile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:701` — promises.readFile → `expect(await fs.promises.readFile(import.meta.path, "utf-8")).toEqual(readFileSync(import.meta.path, "utf-8"))`
- `test/js/node/fs/fs.test.ts:702` — promises.readFile → `expect(await fs.promises.readFile(import.meta.path, { encoding: "latin1" })).toEqual( readFileSync(import.meta.path, { encoding: "latin1" }), )`
- `test/js/node/fs/fs.test.ts:967` — promises.readFile - UTF16 file path → `expect(await fs.promises.readFile(dest, "utf-8")).toEqual(expected)`

## FS21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.readFile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/patch/patch.test.ts:102` — apply > edgecase → `expect(await fs.readFile('${tempdir}/node_modules/is-even/index.js').then(b => b.toString())).toBe(newcontents)`
- `test/js/bun/bun-object/write.spec.ts:75` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(await fs.readFile(filepath, "utf-8")).toBe(content)`
- `test/js/bun/bun-object/write.spec.ts:81` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(await fs.readFile(filepath, "utf-8")).toBe("")`

## FS22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.readFileSync** — exhibits the property captured in the witnessing test. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/init/init.test.ts:33` — bun init works → `expect(readme).toStartWith("# " + path.basename(temp).toLowerCase().replaceAll(" ", "-") + "\n")`
- `test/cli/init/init.test.ts:34` — bun init works → `expect(readme).toInclude("v" + Bun.version.replaceAll("-debug", ""))`
- `test/cli/init/init.test.ts:35` — bun init works → `expect(readme).toInclude("index.ts")`

## FS23
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fs.readdirSync** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/24007.test.ts:102` — issue #24007 - glob with recursive patterns > fs.readdirSync with recursive option finds a… → `expect(results).toContain("file.txt")`
- `test/cli/run/glob-on-fuse.test.ts:79` — fs.readdirSync works on FUSE mount → `expect(results).toContain("main.js")`
- `test/regression/issue/24007.test.ts:103` — issue #24007 - glob with recursive patterns > fs.readdirSync with recursive option finds a… → `expect(results).toContain(path.join("a", "file.txt"))`

## FS24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.stat** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/bun-object/write.spec.ts:83` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(stats.mode & default_mode).toBe(default_mode)`
- `test/js/bun/bun-object/write.spec.ts:84` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(stats.mode & constants.S_IFDIR).toBe(0)`
- `test/js/bun/bun-object/write.spec.ts:92` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(stats.mode & default_mode).toBe(default_mode)`

