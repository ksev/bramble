import { derived, type Readable } from "svelte/store";

export interface RpcTransport {
    unary(serviceId: number, callId: number, input: Uint8Array): Promise<Uint8Array>;
    stream(serviceId: number, callId: number, input: Uint8Array): Readable<Uint8Array>;
}
