import { derived, writable, type Readable, type Writable } from "svelte/store";
import Api, { type ApiClient, type Value } from '$data/api';
import type { Device } from "./api-gen/api_types";
import { identity } from "svelte/internal";

const valueStore = new Map<string, Writable<Value>>();

const setValue = (device: string, feature: string, value: Value) => {
    const key = `${device}/${feature}`;
    if (!valueStore.has(key)) {
        valueStore.set(key, writable(value));
    } else {
        valueStore.get(key).set(value);
    }
}

export const value = (device: string, property: string) => {
    const key = `${device}/${property}`;

    if (!valueStore.has(key)) {
        valueStore.set(key, writable({ value: null }));
    }

    return {
        subscribe: valueStore.get(key).subscribe
    }
}

class Devices {
    private deviceMap = new Map<string, Device>();
    private deviceList = writable<Device[]>([]);

    private initialized = false;

    constructor() {
        Api.subscribe(c => {
            if (c === undefined) return;
            this.fullSync(c);
        });
    }

    all = (): Readable<Device[]> => {
        return derived(this.deviceList, identity);
    }

    byId = (id: string): Promise<Device> => {
        if (this.initialized) {
            const dev = this.deviceMap.get(id);
            return Promise.resolve(dev);
        }

        return new Promise((resolve) => {
            const unsub = this.deviceList.subscribe(_ => {
                // We only have a valid "request" if we have other devices in the list 
                // which means we have initialized
                if (this.initialized) {
                    resolve(this.deviceMap.get(id));
                    unsub();
                }
            })
        });
    }

    private async fullSync(client: ApiClient) {
        const result = await client.getAllDevices();

        for (const device of result.device) {
            this.deviceMap.set(device.id, device);

            for (const feature of device.features) {
                if (!feature.value) continue;
                setValue(device.id, feature.id, feature.value);
            }
        }

        this.initialized = true;
        this.deviceList.set(result.device);  
        
        this.subscribeDeviceUpdates(client);
        this.subscribeFeatureValues(client);
    }

    private async subscribeDeviceUpdates(client: ApiClient) {
        for await (const result of client.deviceUpdates()) {
            const device = result.device;
            this.deviceMap.set(device.id, device);
            this.deviceList.set(Array.from(this.deviceMap.values()));
        }
    }

    private async subscribeFeatureValues(client: ApiClient) {
        for await (const result of client.valueUpdates()) {
            const data = result.values;
            setValue(data.device, data.feature, data.value);
        }
    }
}

export const devices = new Devices();

/*
devices.all() -> Readable<Device[]>
devices.byType(DeviceType.Hardware, DeviceType.Virtual) -> Readable<Device[]>
devices.byId('xc0') -> Promise<Device>
devices.hasParent('xc0') -> Readable<Device[]>
*/

