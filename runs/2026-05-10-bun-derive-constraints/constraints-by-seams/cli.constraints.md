# @cli — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: cli-surface-property
  threshold: CLI1
  interface: [Bun.file]

@imports: []

@pins: []

Surface drawn from 18 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 17. Total witnessing constraint clauses: 294.

## CLI1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/update_interactive_formatting.test.ts:1394` — bun update --interactive > should handle version ranges with multiple conditions → `expect(packageJson.catalog["no-deps"]).toBeDefined()`
- `test/cli/update_interactive_formatting.test.ts:1395` — bun update --interactive > should handle version ranges with multiple conditions → `expect(packageJson.catalog["dep-with-tags"]).toBeDefined()`

## CLI2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.requested** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 114)

Witnessed by 114 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:231` — bun-install > should not error when package.json has comments and trailing commas → `expect(ctx.requested).toBe(1)`
- `test/cli/install/bun-install.test.ts:632` — bun-install > should handle missing package → `expect(ctx.requested).toBe(1)`
- `test/cli/install/bun-install.test.ts:685` — bun-install > should handle @scoped authentication → `expect(ctx.requested).toBe(1)`

## CLI3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.package_dir** — exhibits the property captured in the witnessing test. (behavioral; cardinality 28)

Witnessed by 28 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:815` — bun-install > should handle workspaces → `expect(ctx.package_dir).toHaveWorkspaceLink(["Bar", "bar"])`
- `test/cli/install/bun-install.test.ts:816` — bun-install > should handle workspaces → `expect(ctx.package_dir).toHaveWorkspaceLink(["Asterisk", "packages/asterisk"])`
- `test/cli/install/bun-install.test.ts:817` — bun-install > should handle workspaces → `expect(ctx.package_dir).toHaveWorkspaceLink(["AsteriskTheSecond", "packages/second-asterisk"])`

## CLI4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**update.exited** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 25)

Witnessed by 25 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/update_interactive_formatting.test.ts:412` — bun update --interactive > should update packages when 'a' (select all) is used → `expect(exitCode).toBe(0)`
- `test/cli/update_interactive_formatting.test.ts:478` — bun update --interactive > should handle workspace updates with recursive flag → `expect(exitCode).toBe(0)`
- `test/cli/update_interactive_formatting.test.ts:537` — bun update --interactive > should handle catalog updates correctly → `expect(exitCode).toBe(0)`

## CLI5
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**r.stdout.search** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/run/multi-run.test.ts:487` — sequential: basic > runs scripts in order → `expect(i1).toBeGreaterThan(-1)`
- `test/cli/run/multi-run.test.ts:488` — sequential: basic > runs scripts in order → `expect(i2).toBeGreaterThan(-1)`
- `test/cli/run/multi-run.test.ts:489` — sequential: basic > runs scripts in order → `expect(i3).toBeGreaterThan(-1)`

## CLI6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**out.replace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 4 test files. Antichain representatives:

- `test/cli/install/bun-workspaces.test.ts:99` — dependency on workspace without version in package.json → `expect(out.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "2 packages installed", ])`
- `test/cli/install/bun-link.test.ts:63` — should link and unlink workspace package → `expect(out.replace(/\s*\[[0-9\.]+ms\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "Done! Checked 3 packages (no changes)", ])`
- `test/cli/install/bun-install-registry.test.ts:2286` — --production excludes devDependencies in workspaces → `expect(out.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", expect.stringContaining("+ no-deps@1.0.0"), "", "4 packages installed", ])`

## CLI7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**getOutput.trim** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-pm-pkg.test.ts:219` — bun pm pkg > set command > should set a simple string property → `expect(getOutput.trim()).toBe('"New description"')`
- `test/cli/install/bun-pm-pkg.test.ts:236` — bun pm pkg > set command > should set nested properties with dot notation → `expect(getOutput.trim()).toBe('"echo hello"')`
- `test/cli/install/bun-pm-pkg.test.ts:253` — bun pm pkg > set command > should handle JSON boolean true with --json flag → `expect(getOutput.trim()).toBe("true")`

## CLI8
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**files.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/run/cpu-prof.test.ts:42` — --cpu-prof > generates CPU profile with default name → `expect(profileFiles.length).toBeGreaterThan(0)`
- `test/cli/env/bun-options.test.ts:79` — BUN_OPTIONS environment variable > bare flag before flag with value is recognized → `expect(cpuProfiles.length).toBeGreaterThanOrEqual(1)`
- `test/cli/run/cpu-prof.test.ts:154` — --cpu-prof > --cpu-prof-dir sets custom directory → `expect(profileFiles.length).toBeGreaterThan(0)`

## CLI9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**request.headers.get** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:611` — bun-install > should handle missing package → `expect(request.headers.get("accept")).toBe( "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*", )`
- `test/cli/install/bun-add.test.ts:403` — should handle semver-like names → `expect(request.headers.get("accept")).toBe( "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*", )`
- `test/cli/install/bun-install.test.ts:614` — bun-install > should handle missing package → `expect(request.headers.get("npm-auth-type")).toBe(null)`

## CLI10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**err.split** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-link.test.ts:62` — should link and unlink workspace package → `expect(err.split(/\r?\n/).slice(-2)).toEqual(["Saved lockfile", ""])`
- `test/cli/install/bun-link.test.ts:79` — should link and unlink workspace package → `expect(err.split(/\r?\n/)).toEqual([""])`
- `test/cli/install/bun-link.test.ts:93` — should link and unlink workspace package → `expect(err.split(/\r?\n/)).toEqual([""])`

## CLI11
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**r.stdout.indexOf** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/run/multi-run.test.ts:957` — timing edge cases > sequential: rapid scripts complete in order → `expect(ia).toBeLessThan(ib)`
- `test/cli/run/multi-run.test.ts:958` — timing edge cases > sequential: rapid scripts complete in order → `expect(ib).toBeLessThan(ic)`
- `test/cli/run/multi-run.test.ts:1514` — sequential: status messages between scripts > Done message appears between sequential scri… → `expect(firstIdx).toBeGreaterThan(-1)`

## CLI12
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**result1.stderr.toString** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/test/claudecode-flag.test.ts:92` — CLAUDECODE=1 vs CLAUDECODE=0 comparison → `expect(normalOutput).toContain("(pass)")`
- `test/cli/test/claudecode-flag.test.ts:93` — CLAUDECODE=1 vs CLAUDECODE=0 comparison → `expect(normalOutput).toContain("(skip)")`
- `test/cli/test/claudecode-flag.test.ts:94` — CLAUDECODE=1 vs CLAUDECODE=0 comparison → `expect(normalOutput).toContain("(todo)")`

## CLI13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tests.map** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/inspect/test-reporter.test.ts:293` — retroactively reports tests when TestReporter.enable is called after tests are discovered → `expect(testNames).toEqual(["test A1", "test A2", "test B1"])`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:1355` — Socket Integration - Complex Edge Cases > unicode and special characters in test names → `expect(testNames.some(n => n.includes("\u200B"))).toBe(true)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:1358` — Socket Integration - Complex Edge Cases > unicode and special characters in test names → `expect(testNames.some(n => n.includes("מבחן"))).toBe(true)`

## CLI14
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/run/glob-on-fuse.test.ts:66` — Bun.Glob.scanSync finds files on FUSE mount → `expect(results.length).toBeGreaterThanOrEqual(1)`
- `test/cli/heap-prof.test.ts:27` — --heap-prof generates V8 heap snapshot on exit → `expect(files.length).toBeGreaterThan(0)`
- `test/cli/heap-prof.test.ts:61` — --heap-prof-md generates markdown heap profile on exit → `expect(files.length).toBeGreaterThan(0)`

## CLI15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**CJSArrayLike** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/run/esm-defineProperty.test.ts:23` — arraylike → `expect(CJSArrayLike[0]).toBe(0)`
- `test/cli/run/esm-defineProperty.test.ts:24` — arraylike → `expect(CJSArrayLike[1]).toBe(1)`
- `test/cli/run/esm-defineProperty.test.ts:25` — arraylike → `expect(CJSArrayLike[2]).toBe(3)`

## CLI16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**nameOutput.trim** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-pm-pkg.test.ts:703` — bun pm pkg > npm pkg compatibility tests > should produce equivalent output to npm pkg for… → `expect(nameOutput.trim()).toBe('"test-package"')`
- `test/cli/install/bun-pm-pkg.test.ts:920` — bun pm pkg > fix command > should fix uppercase package names to lowercase → `expect(nameOutput.trim()).toBe('"test-package"')`
- `test/cli/install/bun-pm-pkg.test.ts:981` — bun pm pkg > fix command > should handle package.json with existing bin files → `expect(nameOutput.trim()).toBe('"bin-package"')`

## CLI17
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**node.callFrame** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/run/cpu-prof.test.ts:72` — --cpu-prof > generates CPU profile with default name → `expect(node.callFrame).toHaveProperty("functionName")`
- `test/cli/run/cpu-prof.test.ts:73` — --cpu-prof > generates CPU profile with default name → `expect(node.callFrame).toHaveProperty("scriptId")`
- `test/cli/run/cpu-prof.test.ts:74` — --cpu-prof > generates CPU profile with default name → `expect(node.callFrame).toHaveProperty("url")`

## CLI18
type: specification
authority: derived
scope: module
status: active
depends-on: []

**patchOutput.match** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-install-patch.test.ts:600` — patch > bun patch with --linker=isolated > should create patch for package and commit it → `expect(relativePatchPath).toBeTruthy()`
- `test/cli/install/bun-install-patch.test.ts:663` — patch > bun patch with --linker=isolated > should patch transitive dependency with isolate… → `expect(relativePatchPath).toBeTruthy()`
- `test/cli/install/bun-install-patch.test.ts:716` — patch > bun patch with --linker=isolated > should handle scoped packages with isolated lin… → `expect(relativePatchPath).toBeTruthy()`

