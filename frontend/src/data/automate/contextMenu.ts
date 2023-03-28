import { devices } from "$data/devices";
import { get } from "svelte/store";
import type { Context } from "./automate";
import { alt, deviceNode, equals, isNull } from "./node";
import * as logic from "./nodes/logic";
import * as math from "./nodes/math";

export type Action = 
    { type: "next", fn: () => MenuItem[] } |
    { type: "load", fn: (ctx: Context) => void };

export interface MenuItem {
    text: string,
    icon: string,
    category?: boolean,
    action: Action,
}

const BACK_ITEM: MenuItem = {
    text: "Back",
    icon: "chevrons-left",
    action: {
        type: "next",
        fn: () => ROOT,
    },
};

function* collectDevices() {
    yield BACK_ITEM;

    const d = get(devices.visible());

    for (const device of d) {
        yield {
            text: device.name,
            icon: "cpu",
            action: {
                type: "load",
                fn: (ctx: Context) => ctx.nodes.add(deviceNode(device)),
            },
        } as MenuItem;
    }
}

const LOGICITEMS: MenuItem[] = [
    BACK_ITEM,
    {
        text: "And",
        icon: "logic-and",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.AND),
        },
    },
    {
        text: "Or",
        icon: "logic-or",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.OR),
        },
    },
    {
        text: "Not",
        icon: "logic-not",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.NOT),
        },
    },
    {
        text: "Xor",
        icon: "logic-xor",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.XOR),
        },
    },
    {
        text: "Latch",
        icon: "circuit-switch-open",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.LATCH),
        }
    },
    {
        text: "Toggle",
        icon: "circuit-pushbutton",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(logic.TOGGLE),
        }
    }
];

const MATHITEMS: MenuItem[] = [
    BACK_ITEM,
    {
        text: "Compare",
        icon: "equal-double",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(math.compare()),
        }
    }
    /*
    {
        text: "Compare",
        icon: "equal",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(NUMERIC_OPS.compare),
        }
    },
    {
        text: "Max",
        icon: "math-greater",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(NUMERIC_OPS.max)
        }
    },
    {
        text: "Min",
        icon: "math-lower",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(NUMERIC_OPS.min)
        }
    },
    {
        text: "Bezier transform",
        icon: "vector-bezier-2",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(NUMERIC_OPS.compare),
        }
    },
    */
];

export const ROOT: MenuItem[] = [
    {
        text: "Is null",
        icon: "bolt-off",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(isNull()),
        }
    },
    {
        text: "Equals",
        icon: "equal",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(equals()),
        }
    },
    {
        text: "If",
        icon: "a-b",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(alt()),
        }
    },
    {
        text: "Devices",
        icon: "cpu",
        category: true,
        action: {
            type: "next",
            fn: () => Array.from(collectDevices()),
        },
    },
    {
        text: "Logic",
        icon: "logic-xor",
        category: true,
        action: {
            type: "next",
            fn: () => LOGICITEMS,
        },
    },
    {
        text: "Math",
        icon: "math-symbols",
        category: true,
        action: {
            type: "next",
            fn: () => MATHITEMS,
        },
    },
];