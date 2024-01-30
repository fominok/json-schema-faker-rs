import jsf from "json-schema-faker";

function readInput() {
    const chunkSize = 1024;
    const inputChunks = [];
    let totalBytes = 0;

    // Read all the available bytes
    while (1) {
        const buffer = new Uint8Array(chunkSize);
        const stdin = 0;
        const bytesRead = Javy.IO.readSync(stdin, buffer);

        totalBytes += bytesRead;
        if (bytesRead === 0) {
            break;
        }
        inputChunks.push(buffer.subarray(0, bytesRead));
    }

    // Assemble input into a single Uint8Array
    const { finalBuffer } = inputChunks.reduce((context, chunk) => {
        context.finalBuffer.set(chunk, context.bufferOffset);
        context.bufferOffset += chunk.length;
        return context;
    }, { bufferOffset: 0, finalBuffer: new Uint8Array(totalBytes) });

    return String.fromCharCode.apply(null, finalBuffer);
}

function writeOutput(output) {
    const encodedOutput = new TextEncoder().encode(output);
    const buffer = new Uint8Array(encodedOutput);
    const stdout = 1;
    Javy.IO.writeSync(stdout, buffer);
}

function byteArrayKeywordSub(schema) {
    delete schema.byteArray;
    schema.items = {
        type: "integer",
        minimum: 0,
        maximum: 255
    };
}

jsf.define('byteArray', (value, schema) => {
    byteArrayKeywordSub(schema);
    return schema;
});

const input = JSON.parse(readInput());
const schema = input.schema;
const count = input.count;
const fakeData = Array.from({length: count}, () => jsf.generate(schema));

writeOutput(JSON.stringify(fakeData));
