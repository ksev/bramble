import { devices } from "$data/devices";
import { get } from "svelte/store";
import type { Context } from "./automate";
import { BOOL_LOGIC, deviceNode, isNull, NUMERIC_OPS, STATE_OPS } from "./node";

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
        fn: () => ALL,
    },
};

function* collectDevices() {
    yield BACK_ITEM;

    const d = get(devices.all());

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
            fn: (ctx: Context) => ctx.nodes.add(BOOL_LOGIC.and),
        },
    },
    {
        text: "Or",
        icon: "logic-or",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(BOOL_LOGIC.or),
        },
    },
    {
        text: "Not",
        icon: "logic-not",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(BOOL_LOGIC.not),
        },
    },
    {
        text: "Xor",
        icon: "logic-xor",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(BOOL_LOGIC.xor),
        },
    }
];

const NUMBERSITEMS: MenuItem[] = [
    BACK_ITEM,
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
];

const STATEITEMS: MenuItem[] = [
    BACK_ITEM,
    {
        text: "Compare",
        icon: "equal",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(STATE_OPS.compare(ctx, [])),
        }
    }
];

export const ALL: MenuItem[] = [
    {
        text: "Is null",
        icon: "bolt-off",
        action: {
            type: "load",
            fn: (ctx: Context) => ctx.nodes.add(isNull(ctx)),
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
        text: "State",
        icon: "a-b",
        category: true,
        action: {
            type: "next",
            fn: () => STATEITEMS,
        }
    },
    {
        text: "Numbers",
        icon: "123",
        category: true,
        action: {
            type: "next",
            fn: () => NUMBERSITEMS,
        },
    },
];