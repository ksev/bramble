import Colors from '$data/colors';
import { Extent, Point, Rect } from '$data/geometry';
import { getContext, SvelteComponent } from "svelte";
import { get, writable, type Writable } from "svelte/store";

export const key = Symbol('automate');

export interface Context {
    zoom: Writable<number>,
    pointer: Writable<Point>,
    blockPan: Writable<boolean>,


    layout:  Map<number, LayoutStore>,
    edges: Writable<EdgeData[]>,
    halfEdge: Writable<HalfEdgeData>,
    anchors: (key: IOId) => Writable<Point>,
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

export type IOType = { kind: "boolean" }
                   | { kind: "numeric", min?: number, max?: number }
                   | { kind: "enum", values: string[] }

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

export class IOId {
    constructor(public readonly nodeId: number, public readonly name: string) {}
    toString = () => `${this.nodeId}-${this.name}`
}

export interface EdgeData {
    output: IOId,
    input: IOId,
    type: IOType,
}

export type HalfEdgeData = 
    { output: IOId, type: IOType, over?: IOId } |
    { input: IOId, type: IOType, over?: IOId };

/**
 * Fill in the missing IO for the half edge tuning it into a full "real" edge
 * @param half The current half edge
 * @param missing missing data
 * @returns A full fledged edge
 */
export function completeEdge(half: HalfEdgeData, missing: IOId): EdgeData {
    if ('output' in half) {
        return {
            output: half.output,
            input: missing,
            type: half.type
        };
    } else {
        return {
            input: half.input,
            output: missing,
            type: half.type
        }
    }
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
