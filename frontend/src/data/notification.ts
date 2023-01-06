import { writable, type Writable } from "svelte/store";


export interface NotificationMessage {
    type: "success" | "message" | "error",
    message: string,
}


class Notif {
    private queue: NotificationMessage[] = [];
    private current: Writable<NotificationMessage>

    constructor() {}


}

export default new Notif();