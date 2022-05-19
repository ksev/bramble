import { exponential } from "../data/iterators";
import { debug } from "$data/log";
import { readable } from "svelte/store";

/**
 * A websocket that always tries to be connected
 * Does exponential backoff, the value will either be a number to the next try or a WebSocket instance
 */
export const resocket = (url: string) => readable<number | WebSocket>(0, function start(set) {
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

        ws = new WebSocket(url);
        ws.onopen = open;

        debug(`websocket reconnect in ${delay}ms`);
        
        timeout = setTimeout(retry, delay);
    }

    const open = () => {
        clearTimeout(timeout);
        backOff = exponential(800, 30000);
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