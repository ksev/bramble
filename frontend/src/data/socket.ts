import { exponential } from "./iterators";
import { debug } from "$data/log";
import { readable } from "svelte/store";
import { MutablePromise } from "./utils";

/**
 * A websocket that always tries to be connected
 * Does exponential backoff, the value will either be a number to the next try or a WebSocket instance
 */
export const resocket = (url: string, protocol: string[] = []) => readable<number | WebSocket>(0, function start(set) {
    let ws: WebSocket,
        backOff = exponential(500, 30000),
        timeout: number;

    const retry = () => {
        const delay = backOff.next().value as number;
        set(delay);
        
        if (ws) {
            if (ws.readyState === ws.OPEN) {
                open();
                return;
            }

            ws.onopen = null;
            ws.onclose = null;
            ws.close();
        }

        ws = new WebSocket(url, protocol);
        ws.onopen = open;

        debug(`websocket reconnect in ${delay}ms`);
        
        timeout = setTimeout(retry, delay);
    }

    const open = () => {
        clearTimeout(timeout);
        backOff = exponential(500, 30000);
        ws.onclose = retry;
        set(ws);

        debug('websocket connected');
    }

    retry();

    return () => {
        if (ws) {
            ws.onopen = null;
            ws.onclose = null;
            ws.close();
        }
        clearTimeout(timeout);
    };
});

/// The typed websocket will restrict sending a receivering on the socket to certain types
/// it will also automatically serialize and deserialize json first
export class TypedWebSocket<T, R> {
    constructor(private readonly inner: WebSocket) {}

    /**
     * Send a well typed message 
     * This will be converted to JSON before sending
     * @param data The data to send, must be serializable to json
     */
    send = (data: T) => {
        this.inner.send(JSON.stringify(data));
    }

    /**
     * Listen for messages from the socket
     * @returns An Async iterator that spits our messages from the socket
     */
    listen = (): WebSocketEmitter<R> => new WebSocketEmitter(this.inner);

    /**
     * Close the socket
     * @returns Nothing
     */
    close = () => this.inner.close();
}

/**
 * Turn the event based on message of WebSocket and turn it into an async iterator
 * This class handles back preassure and closes it self nicely when the outer iterator closes or the socket closes
 */
export class WebSocketEmitter<R> implements AsyncIterableIterator<R> {
    private buffer: R[] = [];
    private future: MutablePromise<IteratorResult<R, any>> = undefined;
    private closed = false;

    constructor(private readonly inner: WebSocket) {
        inner.addEventListener('message', this.onMessage);
        inner.addEventListener('close', this.close);
    }

    [Symbol.asyncIterator](): AsyncIterableIterator<R> {
        return this;
    }

    next(): Promise<IteratorResult<R, any>> {
        if (this.buffer.length > 0) {
            const next = this.buffer.shift();
            return Promise.resolve({ done: false, value: next });
        } else if (this.closed) { 
            this.inner.removeEventListener('message', this.onMessage);
            this.inner.removeEventListener('close', this.close);

            return Promise.resolve({ done: true, value: undefined });
        } else {
            this.future = new MutablePromise();
            return this.future;
        }
    }

    return(): Promise<IteratorResult<R, any>> {
        this.closed = true;
        this.inner.removeEventListener('message', this.onMessage);
        this.inner.removeEventListener('close', this.close);

        return Promise.resolve({ done: false, value: undefined });
    }

    private onMessage = (ev: MessageEvent) => {
        const data: R = JSON.parse(ev.data);

        this.buffer.push(data);
   
        if (this.future !== undefined) {
            this.future.resolve({ done: false, value: this.buffer.shift()});
            this.future = undefined;
        }
    }

    close = () => {
        this.closed = true;

        // We have pending data but no new data is coming in the resolve
        if (this.future !== undefined) {
            this.inner.removeEventListener('message', this.onMessage);
            this.inner.removeEventListener('close', this.close);

            this.future.resolve({ done: true, value: undefined });
        }
    }
}