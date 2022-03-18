import Colors from '$data/colors';
import { getContext, SvelteComponent } from "svelte";
import { get, writable, Writable } from "svelte/store";

export const key = Symbol('automate');

export interface Context {
    zoom: Writable<number>,
    blockPan: Writable<boolean>,
    layout:  Map<number, LayoutStore>,
    anchors: (key: [number, string]) => Writable<[number, number]>,
}

export function automateContext(): Context {
    return getContext(key)
}

export interface Input {
    id: string,
    type: IOType,
    label: string,
}

export interface Output {
    id: string,
    type: IOType,
    label: string,
}

export type IOType = { type: "boolean" }
                   | { type: "numeric", min?: number, max?: number }
                   | { type: "enum", values: string[] }

export enum NodeType {
    Source = 0,
    Sink = 1,
    Logic = 2,
}

export const NodeColors = {
    [NodeType.Source]: Colors.source,
    [NodeType.Sink]: Colors.sink,
    [NodeType.Logic]: Colors.icon,
}

export interface NodeData {
    id: number,
    label: string,
    type: NodeType,

    inputs: Input[],
    outputs: Output[],
    settings?: SvelteComponent,
}

export interface NodeLayout {
    x: number,
    y: number,

    width: number,
    height: number,
}

export type LayoutStore = ReturnType<typeof layoutStore>;

export function layoutStore(initialData: NodeLayout) {
    const w = writable(initialData);

    return {
        move: (x: number, y: number) => {  
            let ol = get(w);
 
            let nx = Math.round(x/20) * 20;
            let ny = Math.round(y/20) * 20;

            if (ol.x === nx && ol.y === ny) {
                return;
            }

            w.set({ ...ol, x: nx, y: ny });
        },      
        
        resize: (width: number, height: number) => {
            w.update(l => ({                
                ...l,
                width: Math.ceil(width/20) * 20,
                height: Math.round(height/20) * 20,
            }));
        },
        
        subscribe: w.subscribe,
    }
}
