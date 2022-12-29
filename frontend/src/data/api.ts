import { type DocumentNode, Kind, OperationTypeNode } from "graphql";
import { getSdk } from "./api_types";
import { Ws } from "./ws";

interface ConnectionInitMessage {
    type: 'connection_init';
    payload?: Record<string, unknown>;
}

interface ConnectionAckMessage {
    type: 'connection_ack';
    payload?: Record<string, unknown>;
}

interface SubscribeMessage {
    id: string;
    type: 'subscribe';
    payload: {
        operationName?: string | null;
        query: string;
        variables?: Record<string, unknown> | null;
        extensions?: Record<string, unknown> | null;
    };
}

interface NextMessage {
    id: string;
    type: 'next';
    payload: {
        data: unknown
    };
}

interface ErrorMessage {
    id: string;
    type: 'error';
    payload: unknown[];
}

interface CompleteMessage {
    id: string;
    type: 'complete';
}

interface PingMessage {
    type: 'ping';
    payload?: Record<string, unknown>;
}

interface PongMessage {
    type: 'pong';
    payload?: Record<string, unknown>;
}

type ClientSend = ConnectionInitMessage | PingMessage | PongMessage | CompleteMessage;
type ServerSend = ConnectionAckMessage | PingMessage | PongMessage | NextMessage | ErrorMessage | CompleteMessage;

const readySocket = async (url: string) => {
    const ws = new Ws<ClientSend, ServerSend>(url, ['graphql-transport-ws']);

    await ws.open();

    ws.send({ type: 'connection_init' });

    const ack = await ws.once();

    if (ack.type !== 'connection_ack') {
        throw new Error('Did not get ack from the socket');
    }

    return ws;
}

const client = () => {
    let socket: Promise<Ws<ClientSend, ServerSend>> = null;

    function message(message: ServerSend) {
        console.log(message);

        switch (message.type) {
            case "ping":
                send({ type: "pong" });
                break;
        }
    }

    async function* subscribe(data: Omit<SubscribeMessage, "id">) {
        if (!socket) {
            socket = readySocket('ws://127.0.0.1:8080/api/ws');
        }

        const id = randomId();

        const msg = {
            ...data,
            id,
        };

        const chan = (await socket).listen();

        // I know but it's better if the this is not in the ClientSend
        // makes it easier to not send the wrong thing
        (await socket).send(msg as any);

        for await (const msg of chan) {
            console.log(msg);
            if (msg.type === 'next' && msg.id === id) {
                yield msg.payload.data;
            } else if (msg.type === 'complete' && msg.id === id) {
                return
            }
        }
    }

    async function send(data: ClientSend) {
        if (!socket) {
            socket = readySocket('ws://127.0.0.1:8080/api/ws');
        }
        (await socket).send(data);
    }

    return <V, C, R>(doc: DocumentNode, vars?: V, options?: C): Promise<R> | AsyncIterable<R> => {
        const opdef = doc.definitions.find(d => d.kind === Kind.OPERATION_DEFINITION);

        // We already tests for opdef but this type narrows hence why its here twice
        // it will never hit this second branch
        if (!opdef || opdef.kind !== Kind.OPERATION_DEFINITION) {
            throw new Error('Could not find operation definition');
        }

        switch (opdef.operation) {
            case OperationTypeNode.MUTATION:
            case OperationTypeNode.QUERY:
                return subscribe({
                    type: 'subscribe',
                    payload: {
                        operationName: opdef.name.value,
                        query: doc.loc.source.body,
                        variables: vars as any,
                    }
                }).next().then(o => o.value) as any
            
            case OperationTypeNode.SUBSCRIPTION:
                return subscribe({
                    type: 'subscribe',
                    payload: {
                        operationName: opdef.name.value,
                        query: doc.loc.source.body,
                        variables: vars as any,
                    }
                }) as any
        }
    }
}

function randomId(): string {
    const buffer = new Uint32Array(4);
    crypto.getRandomValues(buffer);
    return base64CompactArrayBuffer(buffer.buffer);
}

function base64CompactArrayBuffer(arrayBuffer: ArrayBuffer) {
    var base64 = ''
    var encodings = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

    var bytes = new Uint8Array(arrayBuffer)
    var byteLength = bytes.byteLength
    var byteRemainder = byteLength % 3
    var mainLength = byteLength - byteRemainder

    var a: number, b: number, c: number, d: number
    var chunk: number;

    // Main loop deals with bytes in chunks of 3
    for (var i = 0; i < mainLength; i = i + 3) {
        // Combine the three bytes into a single integer
        chunk = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2]

        // Use bitmasks to extract 6-bit segments from the triplet
        a = (chunk & 16515072) >> 18 // 16515072 = (2^6 - 1) << 18
        b = (chunk & 258048) >> 12 // 258048   = (2^6 - 1) << 12
        c = (chunk & 4032) >> 6 // 4032     = (2^6 - 1) << 6
        d = chunk & 63               // 63       = 2^6 - 1

        // Convert the raw binary segments to the appropriate ASCII encoding
        base64 += encodings[a] + encodings[b] + encodings[c] + encodings[d]
    }

    // Deal with the remaining bytes and padding
    if (byteRemainder == 1) {
        chunk = bytes[mainLength]

        a = (chunk & 252) >> 2 // 252 = (2^6 - 1) << 2

        // Set the 4 least significant bits to zero
        b = (chunk & 3) << 4 // 3   = 2^2 - 1

        base64 += encodings[a] + encodings[b]
    } else if (byteRemainder == 2) {
        chunk = (bytes[mainLength] << 8) | bytes[mainLength + 1]

        a = (chunk & 64512) >> 10 // 64512 = (2^6 - 1) << 10
        b = (chunk & 1008) >> 4 // 1008  = (2^6 - 1) << 4

        // Set the 2 least significant bits to zero
        c = (chunk & 15) << 2 // 15    = 2^4 - 1

        base64 += encodings[a] + encodings[b] + encodings[c]
    }

    return base64;
}

export default getSdk(client());

