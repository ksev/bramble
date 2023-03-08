<script lang="ts">
    import type { ContextInit } from "$data/automate/automate";
    import { automationTarget, deviceNode, SlotRef, type NodePrototype, isNull, equals } from "$data/automate/node";
    import { AND, NOT, OR, XOR, LATCH, TOGGLE } from "$data/automate/nodes/logic";
    import { devices } from "$data/devices";
    import { Point } from "$data/geometry";
    import Program from "$lib/automate/Program.svelte";

    export let params: {
        deviceid: string;
        property: string;
    };

    let init: ContextInit;
    
    async function resolveDevice() {
        const device = await devices.byId(params.deviceid);
        const feature = device.features.find(f => f.id === params.property);

        if (!feature.automate) {
            init = {
                deviceId: device.id,
                feature: feature.id,
                counter: 1,
                connections: [],
                positions: [[0, new Point(6010-100, 6010)]],
                nodes: [{
                    id: 0,
                    ...automationTarget(device.name, feature)     
                }]
            }
            return;
        }

        const r: Record<string, (p?: any) => Promise<NodePrototype>> = {
            "Target": async () => automationTarget(device.name, feature),
            "And": async () => AND,
            "Or": async () => OR,
            "Not": async () => NOT,
            "Xor": async () => XOR,
            "Toggle": async () => TOGGLE,  
            "Latch": async () => LATCH,
            "Equals": async ({kind, meta}) => equals(kind, meta),
            "IsNull": async (kind) => isNull(kind),
            "Device": async (id) => {
                const target = await devices.byId(id);
                return deviceNode(target); 
            } 
        }

        const nodes = [];
        const positions = [];
        const connections = feature.automate.connections.map(([from, to]) => ({
            from: SlotRef.fromTuple(from),
            to: SlotRef.fromTuple(to)
        }));

        let defaults = new Map(feature.automate.defaults.map(([slot, val]) => {
            const id = SlotRef.fromTuple(slot).toString();
            return [id, val];
        }));

        for (const node of feature.automate.nodes) {
            const [x, y] = node.position;
            positions.push([node.id, new Point(x, y)]);

            if (node.properties.tag in r) {
                const box = {
                    id: node.id,
                    ...await r[node.properties.tag](node.properties.content)
                };

                for (const input of box.inputs) {
                    const id = new SlotRef(node.id, input.id).toString();
                    const value = defaults.get(id);

                    if (value !== undefined) {
                        input.default = value;   
                    }
                }
            
                nodes.push(box);
            } else {
                console.warn(node.properties.tag, "This should be a error node");
            }
        }

        init = {
            deviceId: device.id,
            feature: feature.id,
            counter: feature.automate.counter,
            nodes,
            positions,
            connections
        }
    }

    resolveDevice();
</script>

{#if init}
	<Program initialState={init} />
{/if}
