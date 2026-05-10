# Deno — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: deno-surface-property
  threshold: DENO1
  interface: [Deno.Command, Deno.bundle, Deno.env.get, Deno.serveHttp, Deno.env.delete, Deno.gid, Deno.pid, Deno.ppid, Deno.readDirSync, Deno.uid]

@imports: []

@pins: []

Surface drawn from 33 candidate properties across the Bun test corpus. Construction-style: 10; behavioral (high-cardinality): 23. Total witnessing constraint clauses: 447.

## DENO1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.Command** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 35 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/command_test.ts:502` —  → `assertEquals(output.success, true)`
- `tests/unit/command_test.ts:503` —  → `assertEquals(output.code, 0)`
- `tests/unit/command_test.ts:504` —  → `assertEquals(output.signal, null)`

## DENO2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.bundle** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/bundle_test.ts:80` — bundle: basic in-memory bundle succeeds and returns content → `assertEquals(result.success, true)`
- `tests/unit/bundle_test.ts:81` — bundle: basic in-memory bundle succeeds and returns content → `assertEquals(result.errors.length, 0)`
- `tests/unit/bundle_test.ts:125` — bundle: write to outputDir omits outputFiles and writes files → `assertEquals(result.success, true)`

## DENO3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.env.get** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:19` —  → `assertEquals(r, undefined)`
- `tests/unit/os_test.ts:24` —  → `assertEquals(Deno.env.get("TEST_VAR"), "A")`
- `tests/unit/os_test.ts:26` —  → `assertEquals(Deno.env.get("TEST_VAR"), undefined)`

## DENO4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.serveHttp** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/http_test.ts:240` —  → `assertEquals(await httpConn.nextRequest(), null)`
- `tests/unit/http_test.ts:352` —  → `assertEquals(await httpConn.nextRequest(), null)`

## DENO5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.env.delete** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:25` —  → `assertEquals(Deno.env.delete("TEST_VAR"), undefined)`

## DENO6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.gid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:302` —  → `assertEquals(Deno.gid(), null)`

## DENO7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.pid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:163` —  → `assertEquals(typeof Deno.pid, "number")`

## DENO8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.ppid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:168` —  → `assertEquals(typeof Deno.ppid, "number")`

## DENO9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.readDirSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/read_dir_test.ts:32` —  → `assertEquals(typeof iterator.map, "function")`

## DENO10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.uid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:292` —  → `assertEquals(Deno.uid(), null)`

## DENO11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.statSync** — satisfies the documented invariant. (behavioral; cardinality 81)

Witnessed by 81 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/fs_test.ts:168` — [node/fs writeFileSync] repeated overwrites preserve content on failure → `assert(stat.size > 0, "File should not be empty after repeated writes")`
- `tests/unit_node/_fs/_fs_writeFile_test.ts:353` — Mode is correctly set when writing synchronously → `assert(fileInfo && fileInfo.mode)`
- `tests/unit_node/_fs/_fs_mkdir_test.ts:37` — [node/fs] mkdir mode → `assert(Deno.statSync(tmpDir).mode! & 0o777)`

## DENO12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.statSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 62)

Witnessed by 62 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/fs_test.ts:1464` — [node/fs.link] link accepts Buffer → `assertEquals(Deno.statSync(tempFile), Deno.statSync(linkedFile))`
- `tests/unit_node/_fs/_fs_writeFile_test.ts:354` — Mode is correctly set when writing synchronously → `assertEquals(fileInfo.mode & 0o777, modeSync)`
- `tests/unit_node/_fs/_fs_copy_test.ts:86` — [std/node/fs] cpSync preserveTimestamps copies atime/mtime → `assertEquals(destStat.atime.getTime(), srcStat.atime.getTime())`

## DENO13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.inspect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 36)

Witnessed by 36 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit/url_test.ts:426` —  → `assertEquals( Deno.inspect(url), 'URL { href: "http://example.com/?", origin: "http://example.com", protocol: "http:", username: "", password: "", host: "example.com", hostname: "example.com", port: "…`
- `tests/unit/response_test.ts:78` —  → `assertEquals( Deno.inspect(response), 'Response { body: null, bodyUsed: false, headers: Headers {}, ok: true, redirected: false, status: 200, statusText: "", url: "" }', )`
- `tests/unit/request_test.ts:59` —  → `assertEquals( Deno.inspect(request), 'Request { bodyUsed: false, headers: Headers {}, method: "GET", redirect: "follow", url: "https://example.com/" }', )`

## DENO14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/image_bitmap_test.ts:55` —  → `assertEquals(Deno[Deno.internal].getBitmapData(imageBitmap), new Uint8Array([ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1 ]))`
- `tests/unit/canvas_test.ts:110` —  → `assertEquals(bitmapData.length, expectedData.length)`
- `tests/unit/image_bitmap_test.ts:67` —  → `assertEquals(Deno[Deno.internal].getBitmapData(imageBitmap), new Uint8Array([ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 2, 0, 0, 1, 3, 0, 0, 1, 0, 0, 0, 0, 0,…`

## DENO15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.lstatSync** — satisfies the documented invariant. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/fs_test.ts:504` — [node/fs] fchmod works → `assert(newFileMode && originalFileMode)`
- `tests/unit/symlink_test.ts:19` —  → `assert(newNameInfoLStat.isSymlink)`
- `tests/unit/stat_test.ts:86` —  → `assert(packageInfo.isFile)`

## DENO16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.listenDatagram** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 20)

Witnessed by 20 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/net_test.ts:43` —  → `assertEquals(socket.addr.hostname, "127.0.0.1")`
- `tests/unit/net_test.ts:44` —  → `assertEquals(socket.addr.port, listenPort)`
- `tests/unit/net_test.ts:78` —  → `assertEquals(socket.addr.path, filePath)`

## DENO17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.readFileSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 4 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_handle_test.ts:245` — [node/fs filehandle.truncate] Truncate file with extension → `assertEquals(data, expected)`
- `tests/unit/write_file_test.ts:425` —  → `assertEquals(Deno.readFileSync(filename), new Uint8Array([1, 2]))`
- `tests/unit/truncate_test.ts:15` —  → `assertEquals(Deno.readFileSync(filename).byteLength, 20)`

## DENO18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.readTextFileSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_raw_fd_test.ts:153` — [node/fs] writeSync with position writes at offset without moving cursor → `assertEquals(Deno.readTextFileSync(path), "ABaXYZaaaa")`
- `tests/unit/write_text_file_test.ts:16` —  → `assertEquals(dataRead, "Hello")`
- `tests/unit_node/_fs/_fs_raw_fd_test.ts:174` — [node/fs] async write with position does not move cursor → `assertEquals(Deno.readTextFileSync(path), "ZZaaaXYaaa")`

## DENO19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.stat** — satisfies the documented invariant. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_writeFile_test.ts:203` — Mode is correctly set → `assert(fileInfo && fileInfo.mode)`
- `tests/unit/stat_test.ts:132` —  → `assert(readmeInfo.isFile)`
- `tests/unit_node/_fs/_fs_writeFile_test.ts:232` — Mode is not set when rid is passed → `assert(fileInfo.mode)`

## DENO20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.listenDatagram** — satisfies the documented invariant. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/net_test.ts:42` —  → `assert(socket.addr.transport === "udp")`
- `tests/unit/net_test.ts:77` —  → `assert(socket.addr.transport === "unixpacket")`
- `tests/unit/net_test.ts:114` —  → `assert(socket.addr.transport === "unixpacket")`

## DENO21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.connect** — satisfies the documented invariant. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/net_test.ts:222` —  → `assert(conn.remoteAddr.transport === "tcp")`
- `tests/unit/net_test.ts:225` —  → `assert(conn.localAddr != null)`
- `tests/unit/net_test.ts:257` —  → `assert(conn.remoteAddr.transport === "tcp")`

## DENO22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.connect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/net_test.ts:223` —  → `assertEquals(conn.remoteAddr.hostname, "127.0.0.1")`
- `tests/unit/net_test.ts:224` —  → `assertEquals(conn.remoteAddr.port, listenPort)`
- `tests/unit/net_test.ts:258` —  → `assertEquals(conn.remoteAddr.hostname, "127.0.0.1")`

## DENO23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.core.structuredClone** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `libs/core_testing/unit/serialize_deserialize_test.ts:114` —  → `assertEquals(cloned.test, cloned)`
- `libs/core_testing/unit/serialize_deserialize_test.ts:115` —  → `assertEquals(cloned.test2, circularObject.test2)`
- `libs/core_testing/unit/serialize_deserialize_test.ts:116` —  → `assertEquals(cloned.test3, circularObject.test3)`

## DENO24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.readTextFile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 4 test files. Antichain representatives:

- `tests/unit_node/fs_test.ts:311` — [node/fs createWriteStream → `assertEquals(await Deno.readTextFile(file), "hello, world")`
- `tests/unit_node/_fs/_fs_handle_test.ts:401` — [node/fs filehandle.createWriteStream] Create a write stream → `assertEquals(await Deno.readTextFile(tempFile), "a\n")`
- `tests/unit/net_test.ts:1042` —  → `assertEquals(res, "hello world!")`

## DENO25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.systemMemoryInfo** — satisfies the documented invariant. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:280` —  → `assert(info.total >= 0)`
- `tests/unit/os_test.ts:281` —  → `assert(info.free >= 0)`
- `tests/unit/os_test.ts:282` —  → `assert(info.available >= 0)`

## DENO26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.KvU64** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/kv_test.ts:590` — KvU64 comparison → `assertEquals(a, b)`
- `tests/unit/kv_test.ts:592` — KvU64 comparison → `assertEquals(a, new Deno.KvU64(2n))`
- `tests/unit/kv_test.ts:610` — KvU64 unbox → `assertEquals(a.value, 1n)`

## DENO27
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.lstat** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/stat_test.ts:197` —  → `assert(readmeInfo.isFile)`
- `tests/unit/stat_test.ts:201` —  → `assert(readmeInfoByUrl.isFile)`
- `tests/unit/stat_test.ts:206` —  → `assert(modulesInfo.isSymlink)`

## DENO28
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.realPath** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/real_path_test.ts:66` —  → `assert(realPath.startsWith("/"))`
- `tests/unit/real_path_test.ts:67` —  → `assert(realPath.endsWith(relativePath))`
- `tests/unit/real_path_test.ts:70` —  → `assert(realPath.endsWith(relativePath.replace(/\//g, "\\")))`

## DENO29
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.realPathSync** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/real_path_test.ts:15` —  → `assert(realPath.startsWith("/"))`
- `tests/unit/real_path_test.ts:16` —  → `assert(realPath.endsWith(relative))`
- `tests/unit/real_path_test.ts:19` —  → `assert(realPath.endsWith(relative.replace(/\//g, "\\")))`

## DENO30
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.bundle** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/bundle_test.ts:83` — bundle: basic in-memory bundle succeeds and returns content → `assert(result.outputFiles !== undefined)`
- `tests/unit/bundle_test.ts:128` — bundle: write to outputDir omits outputFiles and writes files → `assert(result.outputFiles == null)`
- `tests/unit/bundle_test.ts:249` — bundle: code splitting with multiple entrypoints → `assert(result.outputFiles !== undefined)`

## DENO31
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.core.getAllLeakTraces** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `libs/core_testing/unit/stats_test.ts:104` —  → `assertEquals(tracesAfter.size, tracesBefore.size + 1)`
- `libs/core_testing/unit/stats_test.ts:131` —  → `assertEquals(tracesAfter.size, tracesBefore.size + 1)`
- `libs/core_testing/unit/stats_test.ts:134` —  → `assertEquals(tracesFinal.size, tracesBefore.size)`

## DENO32
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.core.isBoxedPrimitive** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `libs/core_testing/unit/type_test.ts:52` —  → `assert(Deno.core.isBoxedPrimitive(Object(1n)))`
- `libs/core_testing/unit/type_test.ts:53` —  → `assert(Deno.core.isBoxedPrimitive(new Boolean(true)))`
- `libs/core_testing/unit/type_test.ts:54` —  → `assert(Deno.core.isBoxedPrimitive(new Number(1)))`

## DENO33
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Deno.memoryUsage** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/os_test.ts:312` —  → `assert(typeof mem.rss === "number")`
- `tests/unit/os_test.ts:313` —  → `assert(typeof mem.heapTotal === "number")`
- `tests/unit/os_test.ts:314` —  → `assert(typeof mem.heapUsed === "number")`

