import Colors from '$data/colors';
import { Extent, Point, Rect } from '$data/geometry';
import { getContext, SvelteComponent } from "svelte";
import { get, writable, Writable } from "svelte/store";

export const key = Symbol('automate');

export interface Context {
    zoom: Writable<number>,
    pointer: Writable<Point>,
    blockPan: Writable<boolean>,


    layout:  Map<number, LayoutStore>,
    edges: Writable<EdgeData[]>,
    anchors: (key: [number, string] | "mouse") => Writable<Point>,
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
    Error = 10,
}

export const NodeColors = {
    [NodeType.Source]: Colors.source,
    [NodeType.Sink]: Colors.sink,
    [NodeType.Logic]: Colors.icon,
    [NodeType.Error]: Colors.error,
}

export interface NodeData {
    id: number,
    label: string,
    type: NodeType,

    inputs: Input[],
    outputs: Output[],
    settings?: SvelteComponent,
}

export interface EdgeData {
    from: [number, string] | "mouse",
    to: [number, string] | "mouse",
    type: IOType,
}

export type LayoutStore = ReturnType<typeof layoutStore>;

export function layoutStore(initialData: Rect) {
    const w = writable(initialData);

    return {
        move: (x: number, y: number) => {  
            let current = get(w);

            const origin = new Point(
                Math.round(x/20) * 20,
                Math.round(y/20) * 20,
            );

            if (current.origin.equals(origin)) {
                return;
            }

            w.set(current.moveTo(origin));
        },      
        
        resize: (width: number, height: number) => {
            w.update(current => 
                current.resize(
                    new Extent(
                        Math.ceil(width/20) * 20, 
                        Math.round(height/20) * 20
                    )
                )
            );
        },
        
        subscribe: w.subscribe,
    }
}
