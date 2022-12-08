import type { Device, Result, Value } from "$data/device";
import { derived, writable, type Writable } from "svelte/store";
import { resocket } from "./resocket";

/**
 * The socket we use to communicate with the backend service
 * It re-connects automatically on failure
 */
export const socket = resocket(`ws://${document.domain}:8080/pipe`);

type Publish = any;

export const pipe = derived(socket, ws =>
  typeof ws === 'number' ? undefined : (data: Publish) => ws.send(JSON.stringify(data))
);
