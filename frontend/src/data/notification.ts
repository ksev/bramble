import { get, writable } from "svelte/store";

export interface Notification {
    message: String
    type: 'error' | 'success' | 'notify'
}

const queue = [];

const { subscribe, set } = writable(undefined);
export const current = { subscribe };

function next() {
    const notif = queue.shift();

    if (notif) {
        set(notif);
        setTimeout(next, 2000);
    } else {
        set(undefined);
    }
}

export function success(message: string) {
    push({ type: 'success', message });
}

export function error(message: string) {
    push({ type: 'error', message });
}

export function notify(message: string) {
    push({ type: 'notify', message });
}

function push(notif: Notification) {
    queue.push(notif);
    
    if (!get(current)) {
        next();
    }
}
