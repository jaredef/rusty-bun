// Transformation module. Filters + maps records as a TransformStream.

function makeFilterTransform(predicate) {
    return new TransformStream({
        transform(chunk, controller) {
            if (predicate(chunk)) controller.enqueue(chunk);
        }
    });
}

function makeMapTransform(fn) {
    return new TransformStream({
        transform(chunk, controller) {
            controller.enqueue(fn(chunk));
        }
    });
}

module.exports = { makeFilterTransform, makeMapTransform };
