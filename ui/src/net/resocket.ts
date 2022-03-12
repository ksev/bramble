import { exponential } from "../data/iterators";
import { debug } from "$data/log";
import { readable } from "svelte/store";

/**
 * A websocket that always tries to be connected
 * Does exponential backoff, the value will either be a number to the next try or a WebSocket instance
 */
export const resocket = (url: string) => readable<number | WebSocket>(0, function start(set) {
    let ws = new WebSocket(url),
        backOff = exponential(250, 30000);

    const open = () => {
        backOff = exponential(250, 30000);
        set(ws);

        debug('websocket connected');
    }

    const close = () => {
        const delay = backOff.next().value as number;
        set(delay);
        debug(`websocket reconnect in ${delay}ms`);
        setTimeout(() => {
            if (ws && ws?.readyState === ws?.OPEN) {
                // We are already open, do nothing
                // This might happen if the browser repairs the connection for us
                backOff = exponential(250, 30000);
                return;
            }

            ws.onopen = null;
            ws.onclose = null;

            ws = new WebSocket(url);
            ws.onopen = open;
            ws.onclose = close;
        }, delay);
    }
    
    ws.onopen = open;
    ws.onclose = close;

    return () => {
        // CLOSE, ARE YA KIDDING ME!?
    };
});