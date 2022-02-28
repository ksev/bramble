import { exponential } from "../data/iterators";
import { debug } from "$data/log";

export type ReSocketState =
    | { tag: "connected" }
    | { tag: "closed" }
    | { tag: "connecting"; delay: number };

export class ReSocket {
    private ws: WebSocket;
    private backOff: IterableIterator<number>;

    constructor(
        private url: string,
        private onState: (state: ReSocketState) => void
    ) {
        this.backOff = exponential(250, 30000);
    }

    private connect = () => {
        if (
            this.ws &&
            (this.ws.readyState === this.ws.OPEN ||
                this.ws.readyState === this.ws.CONNECTING)
        ) {
            return;
        }

        this.ws = new WebSocket(this.url);
        this.ws.onopen = this.onopen;
        this.ws.onerror = this.onerror;
    };

    private onopen = () => {
        this.backOff = exponential(250, 30000);

        this.onState({ tag: "connected" });

        debug(`websocket connected ${this.url}`);
    };

    private onerror = () => {
        this.ws.close();

        const delay = this.backOff.next().value;

        this.onState({ tag: "connecting", delay });
        debug(`websocket reconnect in ${delay}ms`);

        setTimeout(() => this.connect(), delay);
    };

    open = () => {
        this.connect();

        this.onState({ tag: "connecting", delay: 0 });
    };

    close = () => {
        this.ws.close();

        debug(`websocket closed ${this.url}`);

        this.onState({ tag: "closed" });
    };
}
