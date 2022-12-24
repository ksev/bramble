import { writable, type Writable } from "svelte/store";
import Api from '$data/api';
import type { Device } from "./api_types";

type Result<T> =
  { ok: T } |
  { err: string };

export const devicesMap = new Map<string, Device>();
const valueStore = new Map<string, Writable<Result<number | string | null | boolean>>>();

const devicesList = writable<Device[]>([]);

export const devices = {
  subscribe: devicesList.subscribe,
}

export const value = (device: string, property: string) => {
  const key = `${device}/${property}`;

  if (!valueStore.has(key)) {
    valueStore.set(key, writable({ ok: null }));
  }

  return {
    subscribe: valueStore.get(key).subscribe
  }
}

async function onLoad() {
  const devices = await (await Api.getAllDevices()).device;

  devicesList.set(devices);

  for (const device of devices) {
    devicesMap.set(device.id, device);
  }

}

onLoad();

