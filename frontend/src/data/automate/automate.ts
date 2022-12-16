import { Extent, Point, Rect } from '$data/geometry';
import * as iter from '$data/iterators';
import { filter, map, pop } from '$data/iterators';
import { getContext, setContext } from "svelte";
import { derived, get, writable, type Readable, type Writable } from "svelte/store";
import { SlotRef, type Connection, type IncompleteConnection, type Node, type NodePrototype, type Slot } from './node';

/*
 * CONTEXT
 **/
export const key = Symbol('automate');

/**
 * Context that exists in the Automation tool and it's sub widgets
 */
export class Context {
    /**
     * Where the context menu should be shown, of null if no menu
     */
    public readonly contextMenu: Writable<Point>
    /**
     * Get the mouse coordinates in the editor space
     */
    public readonly pointer: Readable<Point>
    /**
     * Get the mouse coordinates "view" space 
     */
    public readonly viewPointer: Readable<Point>
    /**
     * Get or set if panning should be disabled 
     */
    public readonly blockPan: Writable<boolean>
    /**
     * A mapping from Node id to the Rectangle the node takes up in editor space
     */
    public readonly layout: Map<number, LayoutStore>
    /*
     * A set of node id that are selected
     */
    public readonly selected: SelectedStore
    /**
     * All individual nodes added to the Automation
     */
    public readonly nodes: NodeStore
    /**
     * All connections made in the Automation
     */
    public readonly connections: ConnectionStore
    /**
     * The start of a new connection that is only connected in one end and tracks the mouse pointer
     */
    public readonly startedConnection: Writable<IncompleteConnection | null>

    private anchorMap = new Map<string, Writable<Point>>;

    constructor(nodes: Node[], layout: NodeLayout[], viewPointer: Readable<Point>, pointer: Readable<Point>) {
        this.contextMenu = writable(null);

        this.pointer = pointer;
        this.viewPointer = viewPointer;

        this.blockPan = writable(false);
        
        this.layout = new Map(iter.map(
            layout, 
            nl => [nl.id, layoutStore(nl.rect)]
        ));

        this.selected = selectedStore(this);
        this.nodes = nodeStore(this, nodes);
        this.connections = connectionStore(this);
        this.startedConnection = writable(null);
    }

    /**
     * Fetch the position in editor space of a slot anchor
     */
    public anchors = (key: SlotRef): Writable<Point> => {
        let w: Writable<Point>;

        let skey = key.toString();

        if (!this.anchorMap.has(skey)) {
            w = writable(Point.ZERO);
            this.anchorMap.set(skey, w);
        } else {
            w = this.anchorMap.get(skey);
        }

        return w;
    }
}

export function automateContext(): Context {
    return getContext(key)
}

export interface NodeLayout {
    id: number,
    rect: Rect,
}

/**
 * Build context for the Automation tool and it's sub widgets
 * ONLY call this from the top level Automate widget
 */
export function buildContext(nodes: Node[], layout: NodeLayout[], viewPointer: Readable<Point>, pointer: Readable<Point>): Context {
    return setContext<Context>(key, new Context(nodes, layout, viewPointer, pointer));
}

export type SelectedStore = ReturnType<typeof selectedStore>;

export interface SelectionMove {
    move: (to: Point) => void,
    hasMoved: () => boolean,
}

export function selectedStore(ctx: Context) {
    const { subscribe, set } = writable(new Set<number>());
    const count = derived({subscribe}, s => s.size);
    
    let isSticky = true;

    return {
        startMove: (from: Point) => {
            const selected = get({ subscribe });

            const origins = new Map(
                map(selected, k => [k, get(ctx.layout.get(k)).origin])
            );

            let hasMoved = false;

            return {
                hasMoved: () => hasMoved,
                move: (to: Point) => {
                    const dx = from.x - to.x;
                    const dy = from.y - to.y;

                    for (const [k, origin] of origins) {
                        ctx.layout.get(k).move(
                            origin.x - dx,
                            origin.y - dy,
                        );
                    }

                    hasMoved = true;
                }
            } as SelectionMove;
        },
        boxSelect: (box: Rect, sticky: boolean = true) => {
            const intersects = map(
                filter(ctx.layout, ([_, store]) => {
                    const nodeBox = get(store);
                    return !!box.intersect(nodeBox);
                }), 
                ([id, _]) => id
            );

            isSticky = sticky;

            set(new Set(intersects));
        },
        isSticky: () => isSticky,
        selectOne: (node: Node, sticky: boolean = true) => {
            isSticky = sticky;
            set(new Set([node.id]));
        },
        count,
        deselectAll: (force: boolean = false) => {
            if (isSticky && !force) {
                return;
            }
            
            set(new Set());
        },
        subscribe, 
    }
}

export type NodeStore = ReturnType<typeof nodeStore>;

export function nodeStore(ctx: Context, start: Node[]) {
    const backend = new Map<number, Node>(map(start, n => [n.id, n]));
    const {subscribe, set} = writable(start);

    let n = 1; // TODO this is just an assumption

    return {
        subscribe,
        add: (node: NodePrototype) => {
            const id = n++;

            let p = get(ctx.pointer);

            // Make sure the Node spawn under the pointer 
            // And is immidiately dragable
            ctx.layout.set(
                id, 
                layoutStore(
                    Rect.numbers(
                        p.x-100, 
                        p.y-15, 
                        200, 
                        200
                    )
                )
            );

            backend.set(id, {
                ...node,
                id: id
            });

            set(Array.from(backend.values()));
        },
        replace: (id: number, node: NodePrototype) => {
            backend.set(id, {
                ...node,
                id: id
            });

            set(Array.from(backend.values()));
        },
        get: (id: number): Node =>  {
            return backend.get(id);
        },
        getSlot: (ref: SlotRef): Slot | undefined => {
            const node = backend.get(ref.nodeId);
            if (!node) return undefined;

            return node.inputs.find(s => s.id === ref.name) ??
                   node.outputs.find(s => s.id === ref.name);
        },
        remove: (nodeId: number) => {
            const node = backend.get(nodeId);
            if (!node || node.target) return;

            backend.delete(nodeId);
            ctx.layout.delete(nodeId);

            for (const input of node.inputs) {
                const ref = new SlotRef(node.id, input.id);   
                const conns = Array.from(ctx.connections.get(ref));

                for (const conn of conns) {
                    ctx.connections.remove(conn);
                }
            }

            for (const output of node.outputs) {
                const ref = new SlotRef(node.id, output.id);            
                const conns = Array.from(ctx.connections.get(ref));

                for (const conn of conns) {
                    ctx.connections.remove(conn);
                }
            }

            set(Array.from(backend.values()));
        },
        onAddedConnection: (connection: Connection) => {
            let {from, to} = connection;

            const cb1 = backend.get(from.nodeId)?.onAddedConnection;
            if (cb1) cb1(from, to);

            const cb2 = backend.get(to.nodeId)?.onAddedConnection;
            if (cb2) cb2(to, from);
        },
        onRemovedConnection: (connection: Connection) => {
            let {from, to} = connection;

            const cb1 = backend.get(from.nodeId)?.onRemovedConnection;
            if (cb1) cb1(from, to);

            const cb2 = backend.get(to.nodeId)?.onRemovedConnection;
            if (cb2) cb2(to, from);
        }
    }
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
            w.update(current =>  {
                return current.resize(
                    new Extent(
                        Math.ceil(width/20) * 20, 
                        Math.ceil(height/20) * 20
                    )
                )
            });
        },
        
        subscribe: w.subscribe,
    }
}

export type ConnectionStore = ReturnType<typeof connectionStore>;

function connectionStore(ctx: Context) {
    const outgoing = new Map<string, Set<string>>();
    const incoming = new Map<string, Set<string>>();

    const {subscribe, set:write} = writable<Connection[]>([]);

    function* get(slot: SlotRef) {
        const sslot = slot.toString();

        if (outgoing.has(sslot)) {
            const remote = outgoing.get(sslot);

            for (const to of remote) {
                yield { from: slot, to: SlotRef.fromString(to) } as Connection
            }
        } else if (incoming.has(sslot)) {
            const remote = incoming.get(sslot);

            for (const from of remote) {
                yield { from: SlotRef.fromString(from), to: slot } as Connection
            }
        }
    }

    function* all() {
        for (const [from, remote] of outgoing) {
            for (const to of remote) {
                yield { from: SlotRef.fromString(from), to: SlotRef.fromString(to)} as Connection;
            }
        }
    }

    return {
        connectionCount: (ref: SlotRef) => {
            const sref = ref.toString();
            const store = outgoing.has(sref) ? outgoing : incoming;

            return derived({subscribe}, _ => {
                return store.get(sref)?.size ?? 0;
            });
        }, 
        add: (connection: Connection) => {
            const { from, to } = connection;

            const sfrom = from.toString();
            const sto = to.toString();

            if (!outgoing.has(sfrom)) {
                outgoing.set(sfrom, new Set([sto]));
            } else {
                const remote = outgoing.get(sfrom);
                remote.add(sto);
            }

            if (!incoming.has(sto)) {
                incoming.set(sto, new Set([sfrom]));
            } else {
                const remote = incoming.get(sto);
                remote.add(sfrom);
            }

            ctx.nodes.onAddedConnection(connection);
    
            // Rebuild the array
            write(Array.from(all()));
        },
        remove: (connection: Connection) => {
            const { from, to } = connection;

            const sfrom = from.toString();
            const sto = to.toString();

            // This should never happend but you know, and a silent error here is fine
            if (outgoing.has(sfrom)) { 
                const remote = outgoing.get(sfrom);
                remote.delete(sto);

                // No more connections here remove the entire set
                if (remote.size === 0) {
                    outgoing.delete(sfrom);
                }

                write(Array.from(all()));
            }

            // This should never happend but you know, and a silent error here is fine
            if (incoming.has(sto)) { 
                const remote = incoming.get(sto);
                remote.delete(sfrom);

                // No more connections here remove the entire set
                if (remote.size === 0) {
                    incoming.delete(sto);
                }

                write(Array.from(all()));
            }

            ctx.nodes.onRemovedConnection(connection);
        },
        remote: (ref: SlotRef): Readable<SlotRef | undefined> => {
            const sref = ref.toString();

            return derived({subscribe}, _ => {
                return pop(map(outgoing.get(sref)?.keys(), SlotRef.fromString)) ?? 
                       pop(map(incoming.get(sref)?.keys(), SlotRef.fromString));
            });
        },
        connected: (slot1: SlotRef, slot2: SlotRef): boolean => {
            const sslot1 = slot1.toString();
            const sslot2 = slot2.toString();

            return outgoing.get(sslot1)?.has(sslot2) ||
                   outgoing.get(sslot2)?.has(sslot1);
        },
        get,
        subscribe
    }
}
