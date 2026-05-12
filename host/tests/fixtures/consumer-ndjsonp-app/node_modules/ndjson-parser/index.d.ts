export declare function createTransformer<T = any>(): TransformStream<Uint8Array, T>;

export declare function parse<T = any>(ndJson: string): T[];

export declare function parseStream<T = any>(stream: ReadableStream<Uint8Array>): Promise<T[]>;

declare const _default: {
    createTransformer: typeof createTransformer;
    parse: typeof parse;
    parseStream: typeof parseStream;
};

export default _default;
