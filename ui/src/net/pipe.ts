import { derived, writable, type Readable } from "svelte/store";
import { ReSocket, type ReSocketState } from "./resocket";

import { debug } from "$data/log";

class ChannelIds {
    private currentId: number = 0;
    private freeStack: number[] = [];

    next = (): number => {
        if (this.freeStack.length > 0) {
            const id = this.freeStack.pop();
            debug(`reuse channel id ${id}`);
            return id;
        }

        const current = this.currentId;

        this.currentId += 1;

        if (this.currentId >= 0xffff) {
            throw new Error("Too many channels open in pipe");
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

function createPipe() {
    const onState = (state: ReSocketState) => {
        w.set(state);
    };

    const idc = new ChannelIds();
    const ws = new ReSocket(`ws://${document.domain}:8080/pipe`, onState);
    const w = writable({ tag: "closed" } as ReSocketState);

    let subs = 0;

    return {
        subscribe: w.subscribe,
        channel: (): Readable<number> => {
            return derived(w, (v, set) => {
                switch (v.tag) {
                    case 'closed':
                        ws.open();
                        set(null);
                        return;

                    case 'connecting': 
                        set(null);
                        return;

                    case 'connected':
                        const id = idc.next();

                        subs++;
                        set(id);

                        return () => {
                            idc.free(id);

                            subs--;

                            if (subs === 0) {
                                ws.close();
                            }
                        }
                }
            });
        },
    };
}

export const pipe = createPipe();
