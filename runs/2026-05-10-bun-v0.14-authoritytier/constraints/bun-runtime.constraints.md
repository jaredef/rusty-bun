# bun-runtime — root constraint set

*Index of surface constraint modules. Each `@imports` entry points at a per-surface document drafted by `derive-constraints invert`. The runtime's contract is the composition of the imported surface properties.*

@provides: bun-runtime-property
  threshold: COMPOSITE
  interface: []

@imports:
  - property: bun-surface-property
    from: path
    path: ./bun.constraints.md
    as: bun
    # witnessing-clauses: 2453
  - property: glob-surface-property
    from: path
    path: ./glob.constraints.md
    as: glob
    # witnessing-clauses: 1307
  - property: yaml-surface-property
    from: path
    path: ./yaml.constraints.md
    as: yaml
    # witnessing-clauses: 774
  - property: util-surface-property
    from: path
    path: ./util.constraints.md
    as: util
    # witnessing-clauses: 576
  - property: fetch-surface-property
    from: path
    path: ./fetch.constraints.md
    as: fetch
    # witnessing-clauses: 467
  - property: buffer-surface-property
    from: path
    path: ./buffer.constraints.md
    as: buffer
    # witnessing-clauses: 457
  - property: path-surface-property
    from: path
    path: ./path.constraints.md
    as: path
    # witnessing-clauses: 375
  - property: json-surface-property
    from: path
    path: ./json.constraints.md
    as: json
    # witnessing-clauses: 357
  - property: json5-surface-property
    from: path
    path: ./json5.constraints.md
    as: json5
    # witnessing-clauses: 322
  - property: fs-surface-property
    from: path
    path: ./fs.constraints.md
    as: fs
    # witnessing-clauses: 255
  - property: reflect-surface-property
    from: path
    path: ./reflect.constraints.md
    as: reflect
    # witnessing-clauses: 244
  - property: sql-surface-property
    from: path
    path: ./sql.constraints.md
    as: sql
    # witnessing-clauses: 242
  - property: structuredclone-surface-property
    from: path
    path: ./structuredclone.constraints.md
    as: structuredclone
    # witnessing-clauses: 227
  - property: url-surface-property
    from: path
    path: ./url.constraints.md
    as: url
    # witnessing-clauses: 219
  - property: array-surface-property
    from: path
    path: ./array.constraints.md
    as: array
    # witnessing-clauses: 210
  - property: response-surface-property
    from: path
    path: ./response.constraints.md
    as: response
    # witnessing-clauses: 174
  - property: textdecoder-surface-property
    from: path
    path: ./textdecoder.constraints.md
    as: textdecoder
    # witnessing-clauses: 146
  - property: crypto-surface-property
    from: path
    path: ./crypto.constraints.md
    as: crypto
    # witnessing-clauses: 128
  - property: uint8array-surface-property
    from: path
    path: ./uint8array.constraints.md
    as: uint8array
    # witnessing-clauses: 125
  - property: promise-surface-property
    from: path
    path: ./promise.constraints.md
    as: promise
    # witnessing-clauses: 119
  - property: markdown-surface-property
    from: path
    path: ./markdown.constraints.md
    as: markdown
    # witnessing-clauses: 110
  - property: object-surface-property
    from: path
    path: ./object.constraints.md
    as: object
    # witnessing-clauses: 106
  - property: process-surface-property
    from: path
    path: ./process.constraints.md
    as: process
    # witnessing-clauses: 85
  - property: date-surface-property
    from: path
    path: ./date.constraints.md
    as: date
    # witnessing-clauses: 78
  - property: url-surface-property
    from: path
    path: ./url.constraints.md
    as: url
    # witnessing-clauses: 77
  - property: number-surface-property
    from: path
    path: ./number.constraints.md
    as: number
    # witnessing-clauses: 62
  - property: atomics-surface-property
    from: path
    path: ./atomics.constraints.md
    as: atomics
    # witnessing-clauses: 60
  - property: error-surface-property
    from: path
    path: ./error.constraints.md
    as: error
    # witnessing-clauses: 56
  - property: request-surface-property
    from: path
    path: ./request.constraints.md
    as: request
    # witnessing-clauses: 53
  - property: worker-surface-property
    from: path
    path: ./worker.constraints.md
    as: worker
    # witnessing-clauses: 53
  - property: events-surface-property
    from: path
    path: ./events.constraints.md
    as: events
    # witnessing-clauses: 42
  - property: readline-surface-property
    from: path
    path: ./readline.constraints.md
    as: readline
    # witnessing-clauses: 37
  - property: stats-surface-property
    from: path
    path: ./stats.constraints.md
    as: stats
    # witnessing-clauses: 36
  - property: stream-surface-property
    from: path
    path: ./stream.constraints.md
    as: stream
    # witnessing-clauses: 34
  - property: performance-surface-property
    from: path
    path: ./performance.constraints.md
    as: performance
    # witnessing-clauses: 32
  - property: set-surface-property
    from: path
    path: ./set.constraints.md
    as: set
    # witnessing-clauses: 31
  - property: string-surface-property
    from: path
    path: ./string.constraints.md
    as: string
    # witnessing-clauses: 31
  - property: dns-surface-property
    from: path
    path: ./dns.constraints.md
    as: dns
    # witnessing-clauses: 31
  - property: event-surface-property
    from: path
    path: ./event.constraints.md
    as: event
    # witnessing-clauses: 30
  - property: statementtype-surface-property
    from: path
    path: ./statementtype.constraints.md
    as: statementtype
    # witnessing-clauses: 28
  - property: atob-surface-property
    from: path
    path: ./atob.constraints.md
    as: atob
    # witnessing-clauses: 28
  - property: buffer-surface-property
    from: path
    path: ./buffer.constraints.md
    as: buffer
    # witnessing-clauses: 26
  - property: file-surface-property
    from: path
    path: ./file.constraints.md
    as: file
    # witnessing-clauses: 23
  - property: x509certificate-surface-property
    from: path
    path: ./x509certificate.constraints.md
    as: x509certificate
    # witnessing-clauses: 23
  - property: s-surface-property
    from: path
    path: ./s.constraints.md
    as: s
    # witnessing-clauses: 22
  - property: websocket-surface-property
    from: path
    path: ./websocket.constraints.md
    as: websocket
    # witnessing-clauses: 22
  - property: any-surface-property
    from: path
    path: ./any.constraints.md
    as: any
    # witnessing-clauses: 21
  - property: abortcontroller-surface-property
    from: path
    path: ./abortcontroller.constraints.md
    as: abortcontroller
    # witnessing-clauses: 21
  - property: a-surface-property
    from: path
    path: ./a.constraints.md
    as: a
    # witnessing-clauses: 20
  - property: blob-surface-property
    from: path
    path: ./blob.constraints.md
    as: blob
    # witnessing-clauses: 20
  - property: urlsearchparams-surface-property
    from: path
    path: ./urlsearchparams.constraints.md
    as: urlsearchparams
    # witnessing-clauses: 20
  - property: http2-surface-property
    from: path
    path: ./http2.constraints.md
    as: http2
    # witnessing-clauses: 19
  - property: csrf-surface-property
    from: path
    path: ./csrf.constraints.md
    as: csrf
    # witnessing-clauses: 18
  - property: map-surface-property
    from: path
    path: ./map.constraints.md
    as: map
    # witnessing-clauses: 16
  - property: webassembly-surface-property
    from: path
    path: ./webassembly.constraints.md
    as: webassembly
    # witnessing-clauses: 16
  - property: btoa-surface-property
    from: path
    path: ./btoa.constraints.md
    as: btoa
    # witnessing-clauses: 16
  - property: os-surface-property
    from: path
    path: ./os.constraints.md
    as: os
    # witnessing-clauses: 16
  - property: headers-surface-property
    from: path
    path: ./headers.constraints.md
    as: headers
    # witnessing-clauses: 15
  - property: vm-surface-property
    from: path
    path: ./vm.constraints.md
    as: vm
    # witnessing-clauses: 15
  - property: server-surface-property
    from: path
    path: ./server.constraints.md
    as: server
    # witnessing-clauses: 14
  - property: proxy-surface-property
    from: path
    path: ./proxy.constraints.md
    as: proxy
    # witnessing-clauses: 13
  - property: abortsignal-surface-property
    from: path
    path: ./abortsignal.constraints.md
    as: abortsignal
    # witnessing-clauses: 12
  - property: macro-params-surface-property
    from: path
    path: ./macro-params.constraints.md
    as: macro_params
    # witnessing-clauses: 12
  - property: mimetype-surface-property
    from: path
    path: ./mimetype.constraints.md
    as: mimetype
    # witnessing-clauses: 12

@pins: []

## COMPOSITE
type: bridge
authority: derived
scope: system
status: active
depends-on: []

The Bun runtime contract is composed of 89 surface modules drafted from the test corpus. Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error), target-language derivation operates over this composition; the constraint set is the durable artifact and target-language implementations are ephemeral cache.

Top surfaces by witnessing-clause count:

- **Bun** — 2453 clauses
- **Glob** — 1307 clauses
- **YAML** — 774 clauses
- **util** — 576 clauses
- **fetch** — 467 clauses
- **Buffer** — 457 clauses
- **path** — 375 clauses
- **JSON** — 357 clauses
- **JSON5** — 322 clauses
- **fs** — 255 clauses
- **Reflect** — 244 clauses
- **SQL** — 242 clauses
- **structuredClone** — 227 clauses
- **URL** — 219 clauses
- **Array** — 210 clauses
- **Response** — 174 clauses
- **TextDecoder** — 146 clauses
- **crypto** — 128 clauses
- **Uint8Array** — 125 clauses
- **Promise** — 119 clauses
