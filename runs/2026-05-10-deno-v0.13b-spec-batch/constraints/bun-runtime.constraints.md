# bun-runtime — root constraint set

*Index of surface constraint modules. Each `@imports` entry points at a per-surface document drafted by `derive-constraints invert`. The runtime's contract is the composition of the imported surface properties.*

@provides: bun-runtime-property
  threshold: COMPOSITE
  interface: []

@imports:
  - property: deno-surface-property
    from: path
    path: ./deno.constraints.md
    as: deno
    # witnessing-clauses: 322
  - property: url-surface-property
    from: path
    path: ./url.constraints.md
    as: url
    # witnessing-clauses: 170
  - property: fetch-surface-property
    from: path
    path: ./fetch.constraints.md
    as: fetch
    # witnessing-clauses: 79
  - property: textdecoder-surface-property
    from: path
    path: ./textdecoder.constraints.md
    as: textdecoder
    # witnessing-clauses: 76
  - property: module-surface-property
    from: path
    path: ./module.constraints.md
    as: module
    # witnessing-clauses: 55
  - property: crypto-surface-property
    from: path
    path: ./crypto.constraints.md
    as: crypto
    # witnessing-clauses: 44
  - property: uint8array-surface-property
    from: path
    path: ./uint8array.constraints.md
    as: uint8array
    # witnessing-clauses: 35
  - property: event-surface-property
    from: path
    path: ./event.constraints.md
    as: event
    # witnessing-clauses: 31
  - property: string-surface-property
    from: path
    path: ./string.constraints.md
    as: string
    # witnessing-clauses: 30
  - property: structuredclone-surface-property
    from: path
    path: ./structuredclone.constraints.md
    as: structuredclone
    # witnessing-clauses: 30
  - property: array-surface-property
    from: path
    path: ./array.constraints.md
    as: array
    # witnessing-clauses: 28
  - property: response-surface-property
    from: path
    path: ./response.constraints.md
    as: response
    # witnessing-clauses: 28
  - property: dommatrix-surface-property
    from: path
    path: ./dommatrix.constraints.md
    as: dommatrix
    # witnessing-clauses: 27
  - property: promise-surface-property
    from: path
    path: ./promise.constraints.md
    as: promise
    # witnessing-clauses: 27
  - property: imagedata-surface-property
    from: path
    path: ./imagedata.constraints.md
    as: imagedata
    # witnessing-clauses: 22
  - property: performance-surface-property
    from: path
    path: ./performance.constraints.md
    as: performance
    # witnessing-clauses: 22
  - property: quotaexceedederror-surface-property
    from: path
    path: ./quotaexceedederror.constraints.md
    as: quotaexceedederror
    # witnessing-clauses: 21
  - property: path-surface-property
    from: path
    path: ./path.constraints.md
    as: path
    # witnessing-clauses: 16
  - property: cluster-surface-property
    from: path
    path: ./cluster.constraints.md
    as: cluster
    # witnessing-clauses: 15
  - property: json-surface-property
    from: path
    path: ./json.constraints.md
    as: json
    # witnessing-clauses: 13
  - property: urlsearchparams-surface-property
    from: path
    path: ./urlsearchparams.constraints.md
    as: urlsearchparams
    # witnessing-clauses: 13
  - property: request-surface-property
    from: path
    path: ./request.constraints.md
    as: request
    # witnessing-clauses: 12
  - property: process-surface-property
    from: path
    path: ./process.constraints.md
    as: process
    # witnessing-clauses: 12
  - property: websocket-surface-property
    from: path
    path: ./websocket.constraints.md
    as: websocket
    # witnessing-clauses: 11
  - property: stream-surface-property
    from: path
    path: ./stream.constraints.md
    as: stream
    # witnessing-clauses: 11
  - property: buffer-surface-property
    from: path
    path: ./buffer.constraints.md
    as: buffer
    # witnessing-clauses: 9
  - property: globalthis-surface-property
    from: path
    path: ./globalthis.constraints.md
    as: globalthis
    # witnessing-clauses: 9
  - property: queuemicrotask-surface-property
    from: path
    path: ./queuemicrotask.constraints.md
    as: queuemicrotask
    # witnessing-clauses: 9
  - property: abortsignal-surface-property
    from: path
    path: ./abortsignal.constraints.md
    as: abortsignal
    # witnessing-clauses: 7
  - property: blob-surface-property
    from: path
    path: ./blob.constraints.md
    as: blob
    # witnessing-clauses: 7
  - property: customevent-surface-property
    from: path
    path: ./customevent.constraints.md
    as: customevent
    # witnessing-clauses: 7
  - property: textencoder-surface-property
    from: path
    path: ./textencoder.constraints.md
    as: textencoder
    # witnessing-clauses: 6
  - property: fs-surface-property
    from: path
    path: ./fs.constraints.md
    as: fs
    # witnessing-clauses: 6
  - property: http-surface-property
    from: path
    path: ./http.constraints.md
    as: http
    # witnessing-clauses: 6
  - property: headers-surface-property
    from: path
    path: ./headers.constraints.md
    as: headers
    # witnessing-clauses: 4
  - property: atob-surface-property
    from: path
    path: ./atob.constraints.md
    as: atob
    # witnessing-clauses: 4
  - property: assert-surface-property
    from: path
    path: ./assert.constraints.md
    as: assert
    # witnessing-clauses: 3
  - property: abortcontroller-surface-property
    from: path
    path: ./abortcontroller.constraints.md
    as: abortcontroller
    # witnessing-clauses: 1
  - property: file-surface-property
    from: path
    path: ./file.constraints.md
    as: file
    # witnessing-clauses: 1
  - property: formdata-surface-property
    from: path
    path: ./formdata.constraints.md
    as: formdata
    # witnessing-clauses: 1

@pins: []

## COMPOSITE
type: bridge
authority: derived
scope: system
status: active
depends-on: []

The Bun runtime contract is composed of 40 surface modules drafted from the test corpus. Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), target-language derivation operates over this composition; the constraint set is the durable artifact and target-language implementations are ephemeral cache.

Top surfaces by witnessing-clause count:

- **Deno** — 322 clauses
- **URL** — 170 clauses
- **fetch** — 79 clauses
- **TextDecoder** — 76 clauses
- **module** — 55 clauses
- **crypto** — 44 clauses
- **Uint8Array** — 35 clauses
- **Event** — 31 clauses
- **String** — 30 clauses
- **structuredClone** — 30 clauses
- **Array** — 28 clauses
- **Response** — 28 clauses
- **DOMMatrix** — 27 clauses
- **Promise** — 27 clauses
- **ImageData** — 22 clauses
- **performance** — 22 clauses
- **QuotaExceededError** — 21 clauses
- **path** — 16 clauses
- **cluster** — 15 clauses
- **JSON** — 13 clauses
