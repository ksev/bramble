import Colors, { Color } from '$data/colors';
import type { ValueKind } from '$data/device';
import { Extent, Point, Rect } from '$data/geometry';
import { getContext, setContext } from "svelte";
import { derived, get, writable, type Readable, type Subscriber, type Unsubscriber, type Writable } from "svelte/store";



export interface Slot {
    id: string,
    label: string,
    kind: ValueKind,
    multiple?: boolean,
}

export enum NodeType {
    Source = 0,
    Sink = 1,
    Logic = 2,
    Error = 10,
}

export const NodeColors = {
    [NodeType.Source]: Colors.source,
    [NodeType.Sink]: Colors.sink,
    [NodeType.Logic]: new Color("rgb(146, 139, 225)"),
    [NodeType.Error]: Colors.error,
}

export interface NodeData {
    id: string,
    label: string,
    type: NodeType,
    icon: string,
    target?: boolean,

    inputs: Slot[],
    outputs: Slot[],
    settings?: ConstructorOfATypedSvelteComponent,
}

export class SlotRef {
    constructor(public readonly nodeId: string, public readonly name: string){}
    toString = () => `${this.nodeId}-${this.name}`

    same(other: SlotRef): boolean {
        return this.name === other.name &&
        this.nodeId === other.nodeId;
    }
}

export interface Connection {
    from: SlotRef,
    to: SlotRef,
    kind: ValueKind,
}

export interface IncompleteConnection {
    start: SlotRef,
    startDirection: "input" | "output",
    kind: ValueKind,
    over?: SlotRef
};

/**
 * Create a full connection from an IncompleteConnectio and a SlotRef
 * @param start The start of the incomplete connection
 * @param rest Rest of the data required to complete the connection
 * @returns A connection between two slots
 */
export function completeConnection(start: IncompleteConnection, rest: SlotRef): Connection {
    if (start.startDirection === "input") {
        return {
            to: start.start,
            from: rest,
            kind: start.kind
        };
    } else {
        return {
            to: rest,
            from: start.start,
            kind: start.kind
        }
    }
}

/*
 * CONTEXT
 **/

export const key = Symbol('automate');

/**
 * Context that exists in the Automation tool and it's sub widgets
 */
export interface Context {
    /**
     * Get the zoom level of the editor
     */
    zoom: Readable<number>,
    /**
     * Get the mouse pointer coordinates in the editor space
     */
    pointer: Readable<Point>,
    /**
     * Get or set if panning should be disabled 
     */
    blockPan: Writable<boolean>,
    /**
     * A mapping from Node id to the Rectangle the node takes up in editor space
     */
    layout:  Map<string, LayoutStore>,
    /**
     * All individual nodes added to the Automation
     */
    nodes: Writable<NodeData[]>,
    /**
     * All connections made in the Automation
     */
    connections: ConnectionStore,
    /**
     * The start of a new connection that is only connected in one end and tracks the mouse pointer
     */
    startedConnection: Writable<IncompleteConnection | null>,
    /**
     * Fetch the position in editor space of a slot anchor
     */
    anchors: (key: SlotRef) => Writable<Point>,
}

export function automateContext(): Context {
    return getContext(key)
}

/**
 * Build context for the Automation tool and it's sub widgets
 * ONLY call this from the top level Automate widget
 */
export function buildContext(zoom: Readable<number>, pointer: Readable<Point>): Context {
    const nodes: NodeData[] = [
        {
        id: '0',
        type: NodeType.Source,
        label: "KIT_TMP",
        inputs: [],
        icon: "cpu",
        outputs: [
            {
            id: "temperature_c",
            label: "temperature °C",
            kind: { type: "bool" },
            },
            {
            id: "temperature_f",
            label: "temperature °F",
            kind: { type: "bool" },
            },
        ],
        },

        {
        id: '2',
        type: NodeType.Logic,
        label: "AND",
        icon: "logic-and",
        inputs: [
            {
                id: "inputs",
                label: "inputs",
                multiple: true,
                kind: { type: "bool" },
            },
        ],
        outputs: [
            {
                id: "result",
                label: "output",
                kind: { type: "bool" },
            },
        ],
        },
        {
        id: '1',
        type: NodeType.Sink,
        label: "state",
        target: true,
        icon: "settings-automation",
        inputs: [
            {
            id: "open",
            label: "open %",
            kind: { type: "bool" },
            },
        ],
        outputs: [],
        },
    ];

    const layout = new Map<string, LayoutStore>();

    let i = 0;
    for (const node of nodes) {
        const estHeight = 60 + (node.outputs.length + node.inputs.length) * 20;
        const estWidth = 250 * nodes.length / 2;

        layout.set(
            node.id,
            layoutStore(
                new Rect(
                    new Point(6000 + 250 * i++ - estWidth, 6000 - estHeight / 2),
                    new Extent(0, 0)
                )
            )
        );
    }
    
    const blockPan = writable(false);
    const startedConnection = writable<IncompleteConnection>(null);

    const map = new Map();
    const anchors = (key: SlotRef): Writable<Point> => {
        let w: Writable<Point>;
        let strKey = key.toString();

        if (!map.has(strKey)) {
            w = writable(Point.ZERO);
            map.set(strKey, w);
        } else {
            w = map.get(strKey);
        }

        return w;
    };

    return setContext<Context>(key, {
        zoom,
        blockPan,
        pointer,
        layout,
        anchors,
        nodes: writable(nodes),
        connections: createConnectionStore(),
        startedConnection,
    });
}

/**
 * Contains the Layout data for a Node in the Automation
 * Which is a Rectangle in euclidian space
 */
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

export interface ConnectionStore {
    /**
     * Add a new connection between to Slots
     */
    add: (connection: Connection) => void,
    /**
     * Remove a connection
     */
    remove: (connection: Connection) => void,
    /**
     * Does the connection already exist
     */
    has: (connection: Connection) => boolean,
    /**
     * Get the connections (if any) from a reference, 
     * The reference can be in any end of the connection
     * @param slot The slot to search for 
     */
    get: (slot: SlotRef) => IterableIterator<Connection>

    list: {
        subscribe: (this: void, run: Subscriber<Connection[]>) => Unsubscriber
    }
}

function createConnectionStore(): ConnectionStore {
    const fromTo = new Map<SlotRef, { to: Set<SlotRef>, kind: ValueKind }>();
    const toFrom = new Map<SlotRef, { from: Set<SlotRef>, kind: ValueKind }>();

    const {subscribe, set} = writable([]);

    function* get(slot: SlotRef) {
        if (fromTo.has(slot)) {
            const { to: toSet, kind } = fromTo.get(slot);

            for (const to of toSet) {
                yield { from: slot, to, kind } as Connection
            }
        } else if (toFrom.has(slot)) {
            const { from: fromSet, kind } = toFrom.get(slot);

            for (const from of fromSet) {
                yield { from, to: slot, kind } as Connection
            }
        }
    }

    function* all() {
        for (const [from, {to: toSet, kind}] of fromTo) {

            for (const to of toSet) {
                yield { from, to, kind} as Connection;
            }
        }
    }

    return {
        add: (connection: Connection) => {
            const { from, to, kind } = connection;

            if (!fromTo.has(from)) {
                fromTo.set(from, {
                    to: new Set([to]),
                    kind,
                });
            } else {
                const {to: set, kind: nkind} = fromTo.get(from);

                if (kind.type !== nkind.type) {
                    throw new Error(`Connection added with kind ${nkind.type} does not match already set kind ${kind.type}`);
                }

                set.add(to);
            }

            if (!toFrom.has(to)) {
                toFrom.set(to, {
                    from: new Set([from]),
                    kind,
                });
            } else {
                const s = toFrom.get(to);

                if (s.kind.type !== kind.type) {
                    throw new Error(`Connection added with kind ${kind.type} does not match already set kind ${s.kind.type}`);
                }

                s.from.add(from);
            }

            // Rebuild the array
            set(Array.from(all()));
        },

        remove: (connection: Connection) => {
            const { from, to } = connection;

            // This should never happend but you know, and a silent error here is fine
            if (fromTo.has(from)) { 
                const {to: set} = fromTo.get(from);
                set.delete(to);

                // No more connections here remove the entire set
                if (set.size === 0) {
                    fromTo.delete(from);
                }
            }

            // This should never happend but you know, and a silent error here is fine
            if (toFrom.has(to)) { 
                const {from: set} = toFrom.get(to);
                set.delete(from);

                // No more connections here remove the entire set
                if (set.size === 0) {
                    toFrom.delete(to);
                }
            }

            set(Array.from(all()));
        },

        has: (connection: Connection): boolean => {
            return fromTo.get(connection.from)?.to?.has(connection.to) ?? false
        },

        get,

        list: {
            subscribe
        }
    }
}
