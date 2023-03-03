import { ValueKind } from "$data/api";
import colors from "$data/colors";
import type { NodePrototype } from "../node";

export const AND: NodePrototype = {
    label: "And",
    properties: { tag: "And" },
    icon: "logic-and",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
        multiple: true,
    }],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const OR: NodePrototype = {
    label: "Or",
    properties: { tag: "Or" },
    icon: "logic-or",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
        multiple: true,
    }],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const XOR: NodePrototype = {
    label: "Xor",
    properties: { tag: "Xor" },
    icon: "logic-xor",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
        multiple: true,
    }],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const NOT: NodePrototype = {
    label: "Not",
    properties: { tag: "Not" },
    icon: "logic-not",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
    }],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const LATCH: NodePrototype = {
    label: "Latch",
    properties: { tag: "Latch" },
    icon: "circuit-switch-open",
    color: colors.bool,
    inputs: [
        {
            id: "input",
            label: "Input",
            kind: ValueKind.Bool,
        },
        {
            id: "reset",
            label: "Reset",
            kind: ValueKind.Bool,
        }
    ],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool,
    }]  
};

export const TOGGLE: NodePrototype = {
    label: "Toggle",
    properties: { tag: "Toggle" },
    icon: "circuit-pushbutton",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
    }],
    outputs: [{
        id: "result",
        label: "Result",
        kind: ValueKind.Bool
    }]
};
