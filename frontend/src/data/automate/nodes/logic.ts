import { ValueKind } from "$data/api";
import colors from "$data/colors";
import type { NodePrototype } from "../node";

export const AND: NodePrototype = {
    label: "And",
    properties: "And",
    icon: "logic-and",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
        multiple: true,
    }],
    outputs: [{
        id: "output",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const OR: NodePrototype = {
    label: "Or",
    properties: "Or",
    icon: "logic-or",
    color: colors.bool,
    inputs: [{
        id: "input",
        label: "Input",
        kind: ValueKind.Bool,
        multiple: true,
    }],
    outputs: [{
        id: "output",
        label: "Result",
        kind: ValueKind.Bool,
    }]
};

export const XOR: NodePrototype = {
    label: "Xor",
    properties: "Xor",
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
    properties: "Not",
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
