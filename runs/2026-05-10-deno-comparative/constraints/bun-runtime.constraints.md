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
    # witnessing-clauses: 439
  - property: fetch-surface-property
    from: path
    path: ./fetch.constraints.md
    as: fetch
    # witnessing-clauses: 174
  - property: url-surface-property
    from: path
    path: ./url.constraints.md
    as: url
    # witnessing-clauses: 170
  - property: textdecoder-surface-property
    from: path
    path: ./textdecoder.constraints.md
    as: textdecoder
    # witnessing-clauses: 82
  - property: crypto-surface-property
    from: path
    path: ./crypto.constraints.md
    as: crypto
    # witnessing-clauses: 56
  - property: promise-surface-property
    from: path
    path: ./promise.constraints.md
    as: promise
    # witnessing-clauses: 55
  - property: urlsearchparams-surface-property
    from: path
    path: ./urlsearchparams.constraints.md
    as: urlsearchparams
    # witnessing-clauses: 55
  - property: module-surface-property
    from: path
    path: ./module.constraints.md
    as: module
    # witnessing-clauses: 55
  - property: uint8array-surface-property
    from: path
    path: ./uint8array.constraints.md
    as: uint8array
    # witnessing-clauses: 39
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
  - property: array-surface-property
    from: path
    path: ./array.constraints.md
    as: array
    # witnessing-clauses: 28
  - property: dommatrix-surface-property
    from: path
    path: ./dommatrix.constraints.md
    as: dommatrix
    # witnessing-clauses: 27
  - property: buffer-surface-property
    from: path
    path: ./buffer.constraints.md
    as: buffer
    # witnessing-clauses: 24
  - property: imagedata-surface-property
    from: path
    path: ./imagedata.constraints.md
    as: imagedata
    # witnessing-clauses: 22
  - property: quotaexceedederror-surface-property
    from: path
    path: ./quotaexceedederror.constraints.md
    as: quotaexceedederror
    # witnessing-clauses: 21
  - property: performance-surface-property
    from: path
    path: ./performance.constraints.md
    as: performance
    # witnessing-clauses: 20
  - property: response-surface-property
    from: path
    path: ./response.constraints.md
    as: response
    # witnessing-clauses: 19
  - property: headers-surface-property
    from: path
    path: ./headers.constraints.md
    as: headers
    # witnessing-clauses: 16
  - property: path-surface-property
    from: path
    path: ./path.constraints.md
    as: path
    # witnessing-clauses: 16
  - property: request-surface-property
    from: path
    path: ./request.constraints.md
    as: request
    # witnessing-clauses: 14
  - property: json-surface-property
    from: path
    path: ./json.constraints.md
    as: json
    # witnessing-clauses: 13
  - property: x509certificate-surface-property
    from: path
    path: ./x509certificate.constraints.md
    as: x509certificate
    # witnessing-clauses: 13
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
  - property: structuredclone-surface-property
    from: path
    path: ./structuredclone.constraints.md
    as: structuredclone
    # witnessing-clauses: 11

@pins: []

## COMPOSITE
type: bridge
authority: derived
scope: system
status: active
depends-on: []

The Bun runtime contract is composed of 26 surface modules drafted from the test corpus. Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), target-language derivation operates over this composition; the constraint set is the durable artifact and target-language implementations are ephemeral cache.

Top surfaces by witnessing-clause count:

- **Deno** — 439 clauses
- **fetch** — 174 clauses
- **URL** — 170 clauses
- **TextDecoder** — 82 clauses
- **crypto** — 56 clauses
- **Promise** — 55 clauses
- **URLSearchParams** — 55 clauses
- **module** — 55 clauses
- **Uint8Array** — 39 clauses
- **Event** — 31 clauses
- **String** — 30 clauses
- **Array** — 28 clauses
- **DOMMatrix** — 27 clauses
- **Buffer** — 24 clauses
- **ImageData** — 22 clauses
- **QuotaExceededError** — 21 clauses
- **performance** — 20 clauses
- **Response** — 19 clauses
- **Headers** — 16 clauses
- **path** — 16 clauses
