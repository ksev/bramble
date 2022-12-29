export class Ws<X, R> {

    private inner: WebSocket;

    constructor(url: string, protocol: string[]) {
        this.inner = new WebSocket(url, protocol);
    }

    open = async () => {
        if (this.inner.readyState === this.inner.OPEN) {
            return;
        }

        if (this.inner.readyState === this.inner.CLOSED) {
            throw new Error('Socket is closed');
        }

        await this.onceOrError('open');

        return;
    }

    send = (data: X) => {
        this.inner.send(JSON.stringify(data));
    }

    once = async () => {
        const message = await this.onceOrError('message');
        return JSON.parse(message.data) as R;
    }

    async *listen() {
        const next = await this.once();
        yield next;
    }

    private onceOrError<K extends keyof WebSocketEventMap>(event: K): Promise<WebSocketEventMap[K]> {
        return new Promise((resolve, reject) => {
            const r = (evt: WebSocketEventMap[K]) => {
                this.inner.removeEventListener('error', e);
                resolve(evt);
            }

            const e = (evt: ErrorEvent) => {
                this.inner.removeEventListener(event, e);
                reject(evt.error);
            }

            this.inner.addEventListener(event, r, {once:true});
            this.inner.addEventListener('error', e, {once:true});            
        })
    }
}