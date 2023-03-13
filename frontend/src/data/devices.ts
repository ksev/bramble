import { derived, writable, type Readable, type Writable } from "svelte/store";
import Api, { type ApiClient, type Value } from '$data/api';
import type { Device } from "./api-gen/api_types";
import { identity } from "svelte/internal";
import { MutablePromise } from "./utils";

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

    private initialized = new MutablePromise();

    constructor() {
        Api.subscribe(c => {
            if (c === undefined) return;
            this.fullSync(c);
        });
    }

    all = (): Readable<Device[]> => {
        return derived(this.deviceList, identity);
    }

    byId = async (id: string): Promise<Device> => {
        await this.initialized;
        return this.deviceMap.get(id)
    }

    iter = () => {
        return this.deviceMap.values();
    }

    *children(id: string) {
        for (const d of this.iter()) {
            if (d.parent === id) {
                yield d;
            }
        }
    }

    *byIntegration(name: string) {
        for (const d of this.iter()) {
            if (d.deviceType.name === name) {
                yield d
            }
        }
        
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

        this.initialized.resolve(undefined);
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

