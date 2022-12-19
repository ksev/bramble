import { writable, type Writable } from "svelte/store";
import type { Device, Result, Value } from "./device";

type Message = Device | Value;

export const devicesMap = new Map<string, Device>();
const valueStore = new Map<string, Writable<Result<number | string | null | boolean>>>();

const devicesList = writable<Device[]>([]);

export const devices = {
  subscribe: devicesList.subscribe,
}

export const value = (device: string, property: string) => {
  const key = `${device}/${property}`;

  if (!valueStore.has(key)) {
    valueStore.set(key, writable({ Ok: null }));
  }

  return {
    subscribe: valueStore.get(key).subscribe
  }
}

