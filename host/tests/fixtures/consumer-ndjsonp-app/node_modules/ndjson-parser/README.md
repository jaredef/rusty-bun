# NDJSON Parser

A simple NDJSON (Newline Delimited JSON) parser for Bun, Deno, and Web environments. It provides functions to parse NDJSON strings, streams, and a transformer for real-time processing using `TransformStream`.

## Features

- **String Parsing**: Synchronously parse an NDJSON string into an array of objects.
- **Stream Parsing**: Read a `ReadableStream` (e.g., from `fetch`) and return a promise resolving to an array of objects.
- **TransformStream Support**: Use a dedicated `TransformStream` to process NDJSON chunks as they arrive.
- **Zero Dependencies**: Uses native `TextDecoder` and Web Stream APIs.

## Installation

```bash
bun add ndjson-parser
# or
npm install ndjson-parser
```

## Usage

### 1. Parse an NDJSON String

```javascript
import ndjson from 'ndjson-parser';

const data = `{"id": 1, "name": "foo"}
{"id": 2, "name": "bar"}`;
const result = ndjson.parse(data);

console.log(result);
// Output: [{ id: 1, name: 'foo' }, { id: 2, name: 'bar' }]
```

### 2. Parse a ReadableStream

```javascript
import ndjson from 'ndjson-parser';

const response = await fetch('https://example.com/data.ndjson');
const result = await ndjson.parseStream(response.body);

console.log(result);
```

### 3. Use with TransformStream (Real-time)

```javascript
import ndjson from 'ndjson-parser';

const response = await fetch('https://example.com/data.ndjson');
const transformer = ndjson.createTransformer();

const reader = response.body
  .pipeThrough(transformer)
  .getReader();

while (true) {
  const { value, done } = await reader.read();
  if (done) break;
  console.log('Received object:', value);
}
```

## API

### `parse(ndJsonString)`
Synchronously parses an NDJSON string. Returns an array of objects.

### `parseStream(readableStream)`
Reads a `ReadableStream<Uint8Array>` and returns a `Promise` that resolves to an array of parsed objects.

### `createTransformer()`
Returns a `TransformStream<Uint8Array, any>` that transforms chunks of NDJSON data into parsed JavaScript objects.

## License

MIT
