import { JSONSchemaFaker } from "json-schema-faker";

export function lol() {
    const schema = {
        type: "object",
        properties: {
            user: {
                type: "object",
                properties: {
                    id: {
                        $ref: "#/definitions/positiveInt",
                    },
                    name: {
                        type: "string",
                    },
                    phone: {
                        type: "string",
                        pattern: "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$"
                    },
                    email: {
                        type: "string",
                        format: "email",
                    },
                },
                required: ["id", "name", "email", "phone"],
            },
        },
        required: ["user"],
        definitions: {
            positiveInt: {
                type: "integer",
                minimum: 0,
                exclusiveMinimum: true,
            },
        },
    };

    return JSONSchemaFaker.generate(schema); // [object Object]
}

// Read input from stdin
function readInput() {
    const chunkSize = 1024;
    const inputChunks = [];
    let totalBytes = 0;

    // Read all the available bytes
    while (1) {
        const buffer = new Uint8Array(chunkSize);
        // Stdin file descriptor
        const fd = 0;
        const bytesRead = Javy.IO.readSync(fd, buffer);

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

const input = JSON.parse(readInput());
const schema = input.schema;
const count = input.count;
const fakeData = Array.from({length: count}, () => JSONSchemaFaker.generate(schema));

writeOutput(JSON.stringify(fakeData));
