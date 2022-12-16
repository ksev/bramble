import { socket } from "$net/pipe";
import { writable, type Writable } from "svelte/store";
import type { Device, Result, Value } from "./device";

type Message = Device | Value;

export const devicesMap = new Map<string, Device>;
const valueStore = new Map<string, Writable<Result<number | string | null | boolean>>>;

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

socket.subscribe(ws => {
    if (typeof ws === 'number') return;
  
    ws.onmessage = message => {
      let data: Message = JSON.parse(message.data);
  
      switch (data.type) {
        case 'device': {
          devicesMap.set(data.id, data);
          devicesList.set(Array.from(devicesMap.values()));
          break;
        }
  
        case 'value': {
          const key = `${data.device}/${data.property}`;
  
          if (!valueStore.has(key)) {
            valueStore.set(key, writable(data.value));
          } else {
            valueStore.get(key).set(data.value);
          }
  
          break;
        }
      }
    }
  
    return () => { if (ws) ws.onmessage = null; }
  })