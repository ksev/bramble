import type { Feature } from "$data/api";
import colors, { Color, directionColor, kindColor } from "$data/colors";
import { ValueKind, type Device } from "$data/api";
import { SvelteComponentTyped,  type ComponentProps, type ComponentType } from "svelte";
import { Context } from "./automate";

export interface Node {
    // Monotonic id of node
    id: number,
    // Type of node 
    properties: {
        tag: string,
        content?: any,
    },
    // Node label, shown in the box handle 
    label: string
    // Color of the box handle
    color: Color
    // Icon to show in the box handle
    icon: string
    // Is this node the Automation output
    target?: boolean
    // Node input slots 
    inputs: Slot[]
    // Node output slots 
    outputs: Slot[]
    // Settings component, if the node has one
    settings?: Settings
    // Callback that gets fired when a connection is made 
    onAddedConnection?: (ctx: Context, local: SlotRef, remote: SlotRef) => void
    // Callback that gets fired when a connection is broken
    onRemovedConnection?: (ctx: Context, local: SlotRef, remote: SlotRef) => void
}

/**
 * A utility class to make sure the settings and params do belong togeather but we need 
 * to keep them type erased for rendering
 */
export class Settings {
    private constructor(public readonly component: ComponentType<any>, public readonly props: any) {}

    static fromComponent<C extends SvelteComponentTyped>(component: ComponentType<C>, props: ComponentProps<C>) {
        return new Settings(component, props);
    }
}

export type NodePrototype = Omit<Node, "id">;

export interface Slot {
    id: string,
    label: string,
    kind: ValueKind | "ANY",
    multiple?: boolean,
    default?: string,
    meta?: Record<string, any>
}

export class SlotRef {
    constructor(public readonly nodeId: number, public readonly name: string){}
    
    toString = () => JSON.stringify([this.nodeId, this.name])
    static fromString = (str: string) => {
        const [nodeId, name] = JSON.parse(str);
        return new SlotRef(nodeId, name);
    }

    static fromTuple = ([nodeId, name]: [number, string]) => new SlotRef(nodeId, name);

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
    kind: ValueKind | "ANY",
    over?: SlotRef
};


export const automationTarget = (name: string, feature: Feature): NodePrototype  => ({
    label: name,
    properties: { tag: "Target" },
    color: directionColor(feature.direction),
    icon: "settings-automation",
    target: true,
    inputs: [
        {
            id: feature.id,
            label: `${feature.name}`,
            kind: feature.kind,
            meta: feature.meta,
        },
    ],
    outputs: [],
});

export function isNull(inputKind: ValueKind | "ANY" = "ANY"): NodePrototype {
    return {
        label: "Is null",
        properties: { tag: "IsNull", content: inputKind },
        icon: "bolt-off",
        color: colors.bool,
        onAddedConnection: (ctx, local, remote) => {
            if (local.name === "input") {
                // Specialize to the new possibilities
                const outputSlot = ctx.nodes.getSlot(remote);

                ctx.nodes.replace(
                    local.nodeId, 
                    isNull(outputSlot.kind)
                );
            }
        },
        onRemovedConnection: (ctx, local) => {
            if (local.name === "input") {
                ctx.nodes.replace(
                    local.nodeId, 
                    isNull()
                );
            }
        },
        inputs: [{
            id: "input",
            label: "Input",
            kind: inputKind,
        }],
        outputs: [{
            id: "result",
            label: "Result",
            kind: ValueKind.Bool
        }]
    }
}

export function equals(inputKind: ValueKind | "ANY" = "ANY", meta?: Record<string, any>): NodePrototype {
    const inputs: Slot[] = [{
        id: "input",
        label: "Input",
        kind: inputKind,
    }];

    if (inputKind != "ANY") {
        let def: boolean | string | number;

        switch (inputKind) {
            case ValueKind.Bool:
                def = 'true';
                break;
            case ValueKind.Number:
                def = '10';
                break;
            case ValueKind.State:
            case ValueKind.String:
                def = "";
                break;
        }

        inputs.push({
            id: "other",
            label: "Other",
            kind: inputKind,
            default: def,
            meta,
        });
    }
    
    return {
        label: "Equals",
        properties: { tag: "Equals", content: { kind: inputKind, meta } },
        icon: "equal",
        color: kindColor(inputKind),
        inputs,
        outputs: [{
            id: "result",
            label: "Result",
            kind: ValueKind.Bool,
        }],
        onAddedConnection: (ctx, local, remote) => {
            if (local.name === "input") {
                // Specialize to the new possibilities
                const outputSlot = ctx.nodes.getSlot(remote);

                ctx.nodes.replace(
                    local.nodeId, 
                    equals(outputSlot.kind, outputSlot.meta)
                );
            }
        },
        onRemovedConnection: (ctx, local) => {
            if (local.name === "input") {
                ctx.nodes.replace(
                    local.nodeId, 
                    equals()
                );
            }
        },
    }
}

export function deviceNode(device: Device): NodePrototype {
    const outputs: Slot[] = device
        .features
        .filter(f => f.direction === "SOURCE" || f.direction === "SOURCE_SINK")
        .map(feature => ({
            id: feature.id,
            label: feature.name,
            kind: feature.kind,
            meta: feature.meta,
        }));

    return {
        properties: {
            "tag": "Device",
            "content": device.id,
        },
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