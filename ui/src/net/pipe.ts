import { derived, readable, writable, type Readable, type Writable } from "svelte/store";
import { resocket } from "./resocket";

//import { ConfigServiceClient, Sensor, type RpcTransport } from './protocol';
import { debug, error } from "$data/log";

class ChannelIdGenerator {
    // We start at one beacuse channel zero is reserved for internal messages
    private currentId: number = 1;
    private freeStack: number[] = [];

    next = (): number => {
        if (this.freeStack.length > 0) {
            const id = this.freeStack.pop();
            debug(`reuse channel id ${id}`);
            return id;
        }

        const current = this.currentId;

        this.currentId += 1;

        if (this.currentId > 0x3FFF) {
            throw new Error("Too many open channels in pipe");
        }

        debug(`new channel id ${current}`);

        return current;
    };

    free = (id: number) => {
        debug(`freed channel id ${id}`);
        this.freeStack.push(id);
    };

    reset = () => {
        this.currentId = 0;
        this.freeStack = [];
    };
}

type ChannelCallback = (action: PipeAction, data: Uint8Array) => void;

enum PipeAction {
    Error = 0x0,
    Request = 0x1,
    Response = 0x2,
    Part = 0x3,
    Close = 0x4,
}
/*
class SocketTransport implements RpcTransport {
    private channelCallbacks = new Map<number, ChannelCallback>();
    private idGen = new ChannelIdGenerator();

    constructor(private ws: WebSocket) {
        this.ws.onmessage = (e: MessageEvent<Blob>) => this.onMessage(e.data);
    }

    async unary(callId: number, input: Uint8Array): Promise<Uint8Array> {
        const prefix = new Uint8Array(5); // Prefix is 5 bytes
        const view = new DataView(prefix.buffer)
        const chanId = this.idGen.next();

        view.setUint8(0, PipeAction.Request); // Pipe Action, we want to do a call
        view.setUint16(1, chanId, false); // Channel Id, 
        view.setUint16(3, callId, false); // Which call we want to use

        this.ws.send(new Blob([prefix, input]));

        return new Promise((resolve, reject) => {
            this.channelResponse(chanId, (action, data) => {
                // Clean up before we return with response
                this.idGen.free(chanId);
                this.channelCallbacks.delete(chanId);

                switch (action) {
                    case PipeAction.Response:
                        resolve(data);
                        break;
                    case PipeAction.Error:
                        const txt = new TextDecoder('utf8');
                        reject(new Error(txt.decode(data)));
                        break;
                    default:
                        reject(new Error(`Channel ${chanId} expected Repsonse action got action ${action}`));
                }
            })
        });
    }

    call(callId: number, input: Uint8Array): Readable<Uint8Array> {
        const prefix = new Uint8Array(5); // Prefix is 5 bytes
        const view = new DataView(prefix.buffer)
        const chanId = this.idGen.next();

        view.setUint8(0, PipeAction.Request); // Pipe Action, we want to do a call
        view.setUint16(1, chanId, false); // Channel Id, 
        view.setUint16(3, callId, false); // Which call we want to use

        this.ws.send(new Blob([prefix, input]));

        return readable(undefined, (set) => {
            let cleaned = false;

            const clean = () => {
                if (cleaned) return;
                this.idGen.free(chanId);
                this.channelCallbacks.delete(chanId);
                cleaned = true;
            }

            this.channelResponse(chanId, (action, data) => {
                switch (action) {
                    case PipeAction.Response: {
                        clean();
                        set(data);
                        break;
                    }
                    case PipeAction.Part: {
                        set(data);
                        break;
                    }
                    case PipeAction.Close: {
                        clean();
                    }
                    case PipeAction.Error: {
                        const txt = new TextDecoder('utf8');
                        error(txt.decode(data));
                        set(undefined);
                        clean();
                        break;
                    }
                }
            })

            return clean;
        });
    }

    channelResponse(channelId: number, callback: ChannelCallback) {
        if (this.channelCallbacks.has(channelId)) {
            error(`channelCallback ${channelId} overwritten`);
        }

        this.channelCallbacks.set(channelId, callback);
    }

    onMessage = async (blob: Blob) => {
        const buffer = await blob.arrayBuffer();
        const view = new DataView(buffer);

        const action = view.getUint8(0);
        const channelId = view.getUint16(1, false);

        const callback = this.channelCallbacks.get(channelId);

        if (!callback) {
            error(`Could not find Channel ${channelId} in channelCallbacks`);
            return;
        }

        callback(action, new Uint8Array(buffer).subarray(3));
    }
}



export const configService: SvelteStore<null | ConfigServiceClient>  = derived(socket, (sock, set) => {
    if (typeof sock === 'number') {
        set(null);
        return;
    }

    const transport = new SocketTransport(sock);
    set(new ConfigServiceClient(transport));

    return () => {
        console.log('bye bye');
    }
})

*/

export const socket = resocket(`ws://${document.domain}:8080/pipe`);
