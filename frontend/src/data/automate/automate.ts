import { Extent, Point, Rect } from '$data/geometry';
import { filter, map, pop } from '$data/iterators';
import { getContext, setContext } from "svelte";
import { derived, get, writable, type Readable, type Writable } from "svelte/store";
import { SlotRef, type Connection, type IncompleteConnection, type Node, type NodePrototype, type Slot } from './node';

/*
 * CONTEXT
 **/
export const key = Symbol('automate');

export interface ContextInit {
    counter: number,
    nodes: Node[],
    connections: Connection[],
    positions: [number, Point][],

    deviceId: string,
    feature: string
}

/**
 * Context that exists in the Automation tool and it's sub widgets
 */
export class Context {
    /**
     * Which device we are making a Automation program for
     */
    public readonly deviceId: string
    /**
     * Which feature we are making a Automation program for
     */
    public readonly feature: string
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

    constructor(init: ContextInit, pointer: Readable<Point>, viewPointer: Readable<Point>) {
        this.contextMenu = writable(null);
        this.deviceId = init.deviceId;
        this.feature = init.feature;

        this.pointer = pointer;
        this.viewPointer = viewPointer;

        this.blockPan = writable(false);
        
        this.layout = new Map(map(
            init.positions, 
            ([n, p]) => [n, layoutStore(new Rect(p, new Extent(0, 0)))]
        ));

        this.selected = selectedStore(this);
        this.nodes = nodeStore(this, init);
        this.connections = connectionStore(this, init);
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
export function buildContext(init: ContextInit, pointer: Readable<Point>, viewPointer: Readable<Point>): Context {
    return setContext<Context>(key, new Context(init, pointer, viewPointer));
}

export type SelectedStore = ReturnType<typeof selectedStore>;

export interface SelectionMove {
    move: (to: Point) => void,
    hasMoved: () => boolean,
}

export function selectedStore(ctx: Context) {
    const { subscribe, set, update } = writable(new Set<number>());
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
        select: (node: Node) => {
            update(s => {
                s.add(node.id);
                return s;
            });
        },
        selectAll: () => {
            isSticky = true;
            const ids = map(get(ctx.nodes), n => n.id);
            set(new Set(ids));
        },
        count,
        deselect: (node: Node) => {
            update(s => {
                s.delete(node.id);
                return s;
            });
        },
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

export function nodeStore(ctx: Context, init: ContextInit) {
    const backend = new Map<number, Node>(map(init.nodes, n => [n.id, n]));
    const {subscribe, set} = writable(init.nodes);

    let n = init.counter; 

    return {
        subscribe,
        // We need to save this so we keep id stable over time
        counter: () => n,
        add: (node: NodePrototype, origin?: Point) => {
            const id = n++;

            if (!origin) {
                let p = get(ctx.pointer);

                const x = p.x - 100;
                const y = p.y - 14;

                origin = new Point(x, y);
            } else {
                const x = origin.x - 100;
                const y = origin.y - 14;

                origin = new Point(x, y);
            }

            // Make sure the Node spawn under the pointer 
            // And is immidiatly dragable
            ctx.layout.set(
                id, 
                layoutStore(
                    new Rect(origin, new Extent(200, 200))
                )
            );

            backend.set(id, {
                ...node,
                id: id
            });

            set(Array.from(backend.values()));
        },
        all: () => {
            return Array.from(backend.values());
        },
        replace: (id: number, node: NodePrototype) => {
            const old = backend.get(id);

            backend.set(id, {
                ...node,
                id: id
            });

            set(Array.from(backend.values()));

            // Remove connections that are no longer valid
            // Because the slot arrangement has changed
            if (old) {
                const cb = (a: Slot, b: Slot) => a.id === b.id;
                const inputDiff = diff(old.inputs, node.inputs, cb);
                const outputDiff = diff(old.outputs, node.outputs, cb);

                for (const input of inputDiff.removed) {
                    const ref = new SlotRef(old.id, input.id);   
                    const conns = Array.from(ctx.connections.get(ref));

                    for (const conn of conns) {
                        ctx.connections.remove(conn);
                    }
                }

                for (const output of outputDiff.removed) {
                    const ref = new SlotRef(old.id, output.id);            
                    const conns = Array.from(ctx.connections.get(ref));

                    for (const conn of conns) {
                        ctx.connections.remove(conn);
                    }
                }

                for (const [os, ns] of inputDiff.same) {
                    if (os.kind === ns.kind) continue;

                    const ref = new SlotRef(old.id, os.id);
                    const conns = Array.from(ctx.connections.get(ref));

                    for (const {from, to} of conns) {
                        const n = backend.get(from.nodeId);
                        if (n?.onAddedConnection) n.onAddedConnection.call(n, ctx, from, to);
                    }
                }

                for (const [os, ns] of outputDiff.same) {
                    if (os.kind === ns.kind) continue;

                    const ref = new SlotRef(old.id, os.id);
                    const conns = Array.from(ctx.connections.get(ref));

                    for (const {from, to} of conns) {
                        const n = backend.get(to.nodeId);
                        if (n?.onAddedConnection) n.onAddedConnection.call(n, ctx, to, from);
                    }
                }
            }
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

            const n1 = backend.get(from.nodeId);
            if (n1?.onAddedConnection) n1.onAddedConnection.call(n1, ctx, from, to);

            const n2 = backend.get(to.nodeId);
            if (n2?.onAddedConnection) n2.onAddedConnection.call(n2, ctx, to, from);
        },
        onRemovedConnection: (connection: Connection) => {
            let {from, to} = connection;

            const n1 = backend.get(from.nodeId);
            if (n1?.onRemovedConnection) n1.onRemovedConnection.call(n1, ctx, from, to);

            const n2 = backend.get(to.nodeId);
            if (n2?.onRemovedConnection) n2.onRemovedConnection.call(n2, ctx, to, from);
        }
    }
}

interface Diff<T> {
    added: T[],
    removed: T[],
    same: [T, T][],
}

function diff<T>(before: T[], after: T[], eq: (a: T, b: T) => boolean): Diff<T> {
    const added = [];
    const removed = [];
    const same = [];
    
    for (const old of before) {
        const found = after.find((v) => eq(old, v));

        if (found === undefined) {
            removed.push(old);
        } else {
            same.push([old, found]);
        }
    }

    for (const ne of after) {
        const found = before.find((v) => eq(ne, v));

        if (found === undefined) {
            added.push(ne);
        }
    }

    return { added, removed, same };
}

/**
 * Contains the Layout data for a Node in the Automation
 * Which is a Rectangle in euclidian space
 */
export type LayoutStore = ReturnType<typeof layoutStore>;

export function layoutStore(initialData: Rect) {
    const aligned = new Rect(
        new Point(
            Math.round(initialData.origin.x), 
            Math.round(initialData.origin.y)
        ),
        initialData.size,
    )
    const w = writable(aligned);

    return {
        moveY: (y: number) => {
            w.update(r => {
                const x = r.origin.x;
                return r.moveTo(new Point(x, y));
            });
        },
        moveX: (x: number) => {
            w.update(r => {
                const y = r.origin.y;
                return r.moveTo(new Point(x, y));
            });
        },
        move: (x: number, y: number) => {  
            let current = get(w);

            const origin = new Point(
                Math.round(x),
                Math.round(y),
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
                        Math.ceil(width), 
                        Math.ceil(height)
                    )
                )
            });
        },
        
        subscribe: w.subscribe,
    }
}

export type ConnectionStore = ReturnType<typeof connectionStore>;

function connectionStore(ctx: Context, init: ContextInit) {
    const outgoing = new Map<string, Set<string>>();
    const incoming = new Map<string, Set<string>>();

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

    for (const {from, to} of init.connections) {
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
    }

    
    const {subscribe, set:write} = writable<Connection[]>(Array.from(all()));
    
    return {
        all,
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
        hasConnection: (ref: SlotRef): boolean => {
            const sref = ref.toString();
            return outgoing.has(sref) || incoming.has(sref);  
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
