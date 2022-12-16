import colors, { Color } from "$data/colors";
import type { Device, ValueKind } from "$data/device";
import NumberCompare from "$lib/automate/settings/NumberCompare.svelte";
import StateCompare from "$lib/automate/settings/StateCompare.svelte";
import { SvelteComponentTyped,  type ComponentProps, type ComponentType } from "svelte";
import { Context } from "./automate";

export interface Node {
    id: number,
    label: string,
    color: Color,
    icon: string,
    target?: boolean,

    inputs: Slot[],
    outputs: Slot[],
    settings?: Settings,
   
    onAddedConnection?: (local: SlotRef, remote: SlotRef) => void;
    onRemovedConnection?: (local: SlotRef, remote: SlotRef) => void;
}

/**
 * A utility class to make sure the settings and params do belong togeather but we need 
 * to keep them type erased for rendering
 */
class Settings {
    private constructor(public readonly component: ComponentType<any>, public readonly props: any) {}

    static fromComponent<C extends SvelteComponentTyped>(component: ComponentType<C>, props: ComponentProps<C>) {
        return new Settings(component, props);
    }
}

export type NodePrototype = Omit<Node, "id">;

export interface Slot {
    id: string,
    label: string,
    kind: ValueKind,
    multiple?: boolean,
    default?: number | string | boolean,
}

export class SlotRef {
    constructor(public readonly nodeId: number, public readonly name: string){}
    
    toString = () => JSON.stringify([this.nodeId, this.name])
    static fromString = (str: string) => {
        const [nodeId, name] = JSON.parse(str);
        return new SlotRef(nodeId, name);
    }

    same(other: SlotRef): boolean {
        return this.name === other.name &&
        this.nodeId === other.nodeId;
    }
}

export interface Connection {
    from: SlotRef,
    to: SlotRef,
}

export interface IncompleteConnection {
    start: SlotRef,
    startDirection: "input" | "output",
    kind: ValueKind,
    over?: SlotRef
};

export function isNull(ctx: Context, inputKind: ValueKind = { type: "any" }): NodePrototype {
    return {
        label: "Is null",
        icon: "bolt-off",
        color: colors.bool,
        onAddedConnection: (local, remote) => {
            if (local.name === "input") {
                // Specialize to the new possibilities
                const outputSlot = ctx.nodes.getSlot(remote);

                ctx.nodes.replace(
                    local.nodeId, 
                    isNull(ctx, outputSlot.kind)
                );
            }
        },
        onRemovedConnection: (local) => {
            if (local.name === "input") {
                ctx.nodes.replace(
                    local.nodeId, 
                    isNull(ctx)
                );
            }
        },
        inputs: [{
            id: "input",
            label: "Input",
            kind: inputKind,
        }],
        outputs: [{
            id: "output",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    }
}

export const BOOL_LOGIC: Record<"and" | "or" | "not" | "xor", NodePrototype> = {
    and: {
        label: "And",
        icon: "logic-and",
        color: colors.bool,
        inputs: [{
            id: "input",
            label: "Input",
            kind: {
                type: "bool"
            },
            multiple: true,
        }],
        outputs: [{
            id: "output",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    },
    or: {
        label: "Or",
        icon: "logic-or",
        color: colors.bool,
        inputs: [{
            id: "input",
            label: "Input",
            kind: {
                type: "bool"
            },
            multiple: true,
        }],
        outputs: [{
            id: "output",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    },
    xor: {
        label: "Xor",
        icon: "logic-xor",
        color: colors.bool,
        inputs: [{
            id: "input",
            label: "Input",
            kind: {
                type: "bool"
            },
            multiple: true,
        }],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    },
    not: {
        label: "Not",
        icon: "logic-not",
        color: colors.bool,
        inputs: [{
            id: "input",
            label: "Input",
            kind: {
                type: "bool"
            },
        }],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    }
}

export const NUMERIC_OPS: Record<"compare" | "max" | "min", NodePrototype> = {
    compare: {
        label: "Compare",
        icon: "equal",
        color: colors.number,
        settings: Settings.fromComponent(NumberCompare, {}),
        inputs: [
            {
                id: "a",
                label: "A",
                kind: {
                    type: "number",
                }, 
            },
            {
                id: "b",
                label: "B",
                default: 1,
                kind: {
                    type: "number"
                },
            },
        ],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    },
    max: {
        label: "Max",
        icon: "math-greater",
        color: colors.number,
        inputs: [{
            id: "input",
            label: "Input",
            multiple: true,
            kind: {
                type: "number"
            },
        }],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "number"
            }
        }]
    },
    min: {
        label: "Min",
        icon: "math-less",
        color: colors.number,
        inputs: [{
            id: "input",
            label: "Input",
            multiple: true,
            kind: {
                type: "number"
            },
        }],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "number"
            }
        }]
    },
}

export const STATE_OPS = {
    compare: (ctx: Context, possible: string[]): NodePrototype => ({
        label: "Compare",
        icon: "equal",
        color: colors.state,
        settings: Settings.fromComponent(StateCompare, { possible }),
        onAddedConnection: (local, remote) => {
            if (local.name === "input") {
                // Specialize to the new possibilities
                const outputSlot = ctx.nodes.getSlot(remote);

                if (outputSlot?.kind?.type === "state") {
                    ctx.nodes.replace(
                        local.nodeId, 
                        STATE_OPS.compare(ctx, outputSlot.kind.possible)
                    );
                }
            }
        },
        onRemovedConnection: (local) => {
            if (local.name === "input") {
                ctx.nodes.replace(
                    local.nodeId, 
                    STATE_OPS.compare(ctx, [])
                );
            }
        },
        inputs: [
            {
                id: "input",
                label: "Input",
                kind: {
                    type: "state",
                    possible,
                }, 
            },
        ],
        outputs: [{
            id: "result",
            label: "Result",
            kind: {
                type: "bool"
            }
        }]
    })
}

export function deviceNode(device: Device): NodePrototype {
    const outputs: Slot[] = device
        .features
        .filter(f => f.direction === "source" || f.direction === "sourceSink")
        .map(feature => ({
            id: feature.id,
            label: feature.name,
            kind: feature.kind,
        }));

    return {
        label: device.name,
        color: colors.device,
        icon: "cpu",
        inputs: [],
        outputs
    }
}

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
        };
    } else {
        return {
            to: rest,
            from: start.start,
        }
    }
}