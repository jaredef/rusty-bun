# @cli — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: cli-surface-property
  threshold: CLI1
  interface: [Bun.file]

@imports: []

@pins: []

Surface drawn from 23 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 22. Total witnessing constraint clauses: 624.

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

**out.replace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 241)

Witnessed by 241 constraint clauses across 5 test files. Antichain representatives:

- `test/cli/install/bun-workspaces.test.ts:99` — dependency on workspace without version in package.json → `expect(out.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "2 packages installed", ])`
- `test/cli/install/bun-remove.test.ts:329` — should remove peerDependencies → `expect(out.replace(/\[[0-9\.]+m?s\]/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun remove v1."), "", " done", "", ])`
- `test/cli/install/bun-link.test.ts:63` — should link and unlink workspace package → `expect(out.replace(/\s*\[[0-9\.]+ms\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "Done! Checked 3 packages (no changes)", ])`

## CLI3
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

## CLI4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**urls.sort** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 37)

Witnessed by 37 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:2861` — bun-install > should not reinstall aliased dependencies → `expect(urls.sort()).toBeEmpty()`
- `test/cli/install/bun-add.test.ts:1046` — should add dependency (GitHub) → `expect(urls.sort()).toBeEmpty()`
- `test/cli/install/bun-install.test.ts:3482` — bun-install > should handle GitHub URL in dependencies (user/repo) → `expect(urls.sort()).toBeEmpty()`

## CLI5
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

## CLI6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**out2.replace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 27)

Witnessed by 27 constraint clauses across 5 test files. Antichain representatives:

- `test/cli/install/bun-update.test.ts:117` — should update to latest version of dependency (${input.baz[0]}) → `expect(out2.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun update v1."), "", 'installed baz@${tilde ? "0.0.5" : "0.0.3"} with binaries:', ' - ${tilde ? "b…`
- `test/cli/install/bun-remove.test.ts:133` — should remove existing package → `expect(out2.replace(/ \[[0-9\.]+m?s\]/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun remove v1."), "", "- pkg2", "1 package removed", "", ])`
- `test/cli/install/bun-link.test.ts:225` — should link package → `expect(out2.replace(/\s*\[[0-9\.]+ms\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun link v1."), "", 'installed ${link_name}@link:${link_name}', "", "1 package installed", ])`

## CLI7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**out1.replace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 26)

Witnessed by 26 constraint clauses across 4 test files. Antichain representatives:

- `test/cli/install/bun-update.test.ts:74` — should update to latest version of dependency (${input.baz[0]}) → `expect(out1.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "+ baz@0.0.3", "", "1 package installed", ])`
- `test/cli/install/bun-remove.test.ts:93` — should remove existing package → `expect(out1.replace(/\s*\[[0-9\.]+m?s\]/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun remove v1."), "", '+ pkg2@${pkg2_path.replace(/\\/g, "/")}', "", "1 package installed", "Removed: 1…`
- `test/cli/install/bun-install.test.ts:801` — bun-install > should handle workspaces → `expect(out1.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "4 packages installed", ])`

## CLI8
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

## CLI9
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

## CLI10
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**err.split** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 4 test files. Antichain representatives:

- `test/cli/install/bun-upgrade.test.ts:26` — two invalid arguments, should display error message and suggest command → `expect(err.split(/\r?\n/)).toContain("error: This command updates Bun itself, and does not take package names.")`
- `test/cli/install/bun-install.test.ts:628` — bun-install > should handle missing package → `expect(err.split(/\r?\n/)).toContain('error: GET ${ctx.registry_url}foo - 404')`
- `test/cli/install/bun-create.test.ts:50` — should create selected template with @ prefix → `expect(err.split(/\r?\n/)).toContain( 'error: GET https://registry.npmjs.org/@quick-start%2fcreate-some-template - 404', )`

## CLI11
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

## CLI12
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

## CLI13
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

## CLI14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**err.split** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/install/bun-link.test.ts:62` — should link and unlink workspace package → `expect(err.split(/\r?\n/).slice(-2)).toEqual(["Saved lockfile", ""])`
- `test/cli/install/bun-install.test.ts:4732` — bun-install > should report error on invalid format for optionalDependencies → `expect(err.split("\n")).toEqual([ '1 | {"name":"foo","version":"0.0.1","optionalDependencies":"bar"}', ' ^', 'error: optionalDependencies expects a map of specifiers, e.g.', ' "optionalDependencies": …`
- `test/cli/install/bun-link.test.ts:79` — should link and unlink workspace package → `expect(err.split(/\r?\n/)).toEqual([""])`

## CLI15
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

## CLI16
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

## CLI17
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

## CLI18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**err.trim** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:4812` — bun-install > should report error on duplicated workspace packages → `expect(err.trim().split("\n")).toEqual([ '1 | {"name":"moo","version":"0.0.3"}', ' ^', 'error: Workspace name "moo" already exists', ' at [dir]/baz/package.json:1:9', '', '1 | {"name":"moo","version":…`
- `test/cli/install/bun-install-registry.test.ts:8661` — windows bin linking shim should work > bun run ${bin} arg1 arg2 → `expect(err.trim()).toBe("")`
- `test/cli/install/bun-install-registry.test.ts:8680` — windows bin linking shim should work > bun --bun run ${bin} arg1 arg2 → `expect(err.trim()).toBe("")`

## CLI19
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

## CLI20
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

## CLI21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**out3.replace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-install.test.ts:1412` — bun-install > should handle life-cycle scripts during re-installation → `expect(out3.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "+ qux@0.0.2", "", "2 packages installed", ])`
- `test/cli/install/bun-install.test.ts:1559` — bun-install > should use updated life-cycle scripts in root during re-installation → `expect(out3.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "1 package installed", ])`
- `test/cli/install/bun-install.test.ts:1707` — bun-install > should use updated life-cycle scripts in dependency during re-installation → `expect(out3.replace(/\s*\[[0-9\.]+m?s\]\s*$/, "").split(/\r?\n/)).toEqual([ expect.stringContaining("bun install v1."), "", "1 package installed", ])`

## CLI22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tarball.entries** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/install/bun-pack.test.ts:1280` — bins > basic → `expect(tarball.entries[0].perm & 0o644).toBe(0o644)`
- `test/cli/install/bun-pack.test.ts:1281` — bins > basic → `expect(tarball.entries[1].perm & (0o644 | 0o111)).toBe(0o644 | 0o111)`
- `test/cli/install/bun-pack.test.ts:1315` — bins > directory → `expect(tarball.entries[0].perm & 0o644).toBe(0o644)`

## CLI23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**urls.some** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/install/bunx.test.ts:632` — --package flag > with mock registry > should install specified package when binary differs… → `expect(urls.some(url => url.includes("/my-special-pkg"))).toBe(true)`
- `test/cli/install/bun-install-tarball-integrity.test.ts:693` — should fail (not hang) when registry returns 404 for tarball → `expect(urls.some(u => u.endsWith(".tgz"))).toBe(true)`
- `test/cli/install/bunx.test.ts:671` — --package flag > with mock registry > should support -p shorthand with mock registry → `expect(urls.some(url => url.includes("/actual-package"))).toBe(true)`

