import { derived, writable, type Readable, type Writable } from "svelte/store";
import Api from '$data/api';
import type { Device } from "./api_types";
import { identity } from "svelte/internal";

type Result<T> =
    { ok: T } |
    { err: string };

const valueStore = new Map<string, Writable<Result<number | string | null | boolean>>>();

export const value = (device: string, property: string) => {
    const key = `${device}/${property}`;

    if (!valueStore.has(key)) {
        valueStore.set(key, writable({ ok: null }));
    }

    return {
        subscribe: valueStore.get(key).subscribe
    }
}

class Devices {
    private deviceMap = new Map<string, Device>();
    private deviceList = writable<Device[]>([]);

    private isRunning = false;
    private isInitialized = false;

    constructor() { }

    private init = async () => {
        if (this.isRunning) return;

        // TODO: Error handle this routine

        this.isRunning = true;

        const result = (await Api.getAllDevices()).device;

        for (const device of result) {
            this.deviceMap.set(device.id, device);
        }

        this.isInitialized = true;

        this.deviceList.set(result);   
        
        for await (const result of Api.deviceUpdates()) {
            const device = result.device;
            this.deviceMap.set(device.id, device);
            this.deviceList.set(Array.from(this.deviceMap.values()));
        }
    }

    all = (): Readable<Device[]> => {
        this.init();
        return derived(this.deviceList, identity);
    }
}

export const devices = new Devices();

/*
devices.all() -> Readable<Device[]>
devices.byType(DeviceType.Hardware, DeviceType.Virtual) -> Readable<Device[]>
devices.byId('xc0') -> Promise<Device>
devices.hasParent('xc0') -> Readable<Device[]>
*/

