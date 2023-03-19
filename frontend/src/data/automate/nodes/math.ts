import { Settings, type NodePrototype } from "../node";
import Colors from "$data/colors";
import { ValueKind } from "$data/api";
import NumberCompare from "$lib/automate/settings/NumberCompare.svelte";

export const compare = (operator: string = "Eq"): NodePrototype => ({
    properties: {
        tag: "MathCompare",
    },
    label: "Compare",
    color: Colors.number,
    icon: "equal-double",
    settings: Settings.fromComponent(NumberCompare, { operator }),
    inputs: [
		{
			id: "input",
			label: "Input",
			kind: ValueKind.Number,
		},
		{
			id: "other",
			label: "Other",
			kind: ValueKind.Number,	
			default: '10',
		}
	],
    outputs: [{
		id: "result",
		label: "Result",
		kind: ValueKind.Bool,
	}]
});
