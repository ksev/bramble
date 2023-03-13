import { type DocumentNode, Kind, OperationTypeNode } from "graphql";
import { derived } from "svelte/store";
import { getSdk } from "./api-gen/api_types";
import { asyncFirst, asyncToArray } from "./iterators";
import { error } from "./log";
import { resocket, TypedWebSocket } from "./socket";
import { randomId } from "./utils";

export * from "./api-gen/api_types";

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

interface NextError {
    message: string,
    locations: { line: number, column: number }[],
    path?: string[],
}

interface NextMessage {
    id: string;
    type: 'next';
    payload: { data: any} | { data: null, errors: NextError[] };
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

type ClientSend = ConnectionInitMessage | PingMessage | PongMessage | CompleteMessage | SubscribeMessage;
type ServerSend = ConnectionAckMessage | PingMessage | PongMessage | NextMessage | ErrorMessage | CompleteMessage;

const connectedClient = (socket: TypedWebSocket<ClientSend, ServerSend>) => {
    async function* subscribe(data: Omit<SubscribeMessage, "id">) {
        const id = randomId();
        const msg = { ...data,  id };

        socket.send(msg);

        for await (const msg of socket.listen()) {
            if (msg.type === 'next' && msg.id === id) {
                if ("errors" in msg.payload) {
                    throw new Error(formatErrors(msg.payload.errors));
                }
                yield msg.payload.data;
            } else if (msg.type === 'complete' && msg.id === id) {
                return
            }
        }
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
            case OperationTypeNode.QUERY: {
                let subscription = subscribe({
                    type: 'subscribe',
                    payload: {
                        operationName: opdef.name.value,
                        query: doc.loc.source.body,
                        variables: vars as any,
                    }
                });

                return asyncFirst(subscription);
            }

            case OperationTypeNode.SUBSCRIPTION:
                return subscribe({
                    type: 'subscribe',
                    payload: {
                        operationName: opdef.name.value,
                        query: doc.loc.source.body,
                        variables: vars as any,
                    }
                });
        }
    }
}

function formatErrors(errors: NextError[]) {
    const out = [];

    for (const error of errors) {
        const path = error?.path?.join('/') ?? '';
        const locs = error.locations.map(o => `${o.line}:${o.column}`).join(',');
        out.push(`${path}[${locs}]: ${error.message}`)
    }

    return out.join('\n');
}

function apiHost() {
    if (import.meta.env.DEV) {
        return "ws://127.0.0.1:8080/api/ws";
    }

    const url = new URL("/api/ws", window.location.href);

    url.protocol = url.protocol === "http:" ? "ws:" : "wss:";

    return url.href;
}

export const apiConnection = resocket(apiHost(), ['graphql-transport-ws']);

export type ApiClient = ReturnType<typeof getSdk>;

const client = derived<typeof apiConnection, ApiClient>(apiConnection, (ws, set) => {
    if (typeof ws === 'number') {
        set(undefined);
        return;
    }

    const typed = new TypedWebSocket<ClientSend, ServerSend>(ws);

    typed.send({ type: 'connection_init' });

    /// Initialize the connection the way GraphQl want's it
    (async function() {
        const ack = await asyncFirst(typed.listen());

        if (ack.type !== 'connection_ack') {
            error('Api connection did not get ack response');
            set(undefined);
            return;
        }

        const client = getSdk(connectedClient(typed));
        set(client);
    })();
});

export default client;
