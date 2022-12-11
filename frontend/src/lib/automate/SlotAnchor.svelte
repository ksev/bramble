<script lang="ts">
    import colors from "$data/colors";
    import type { ValueKind } from "$data/device";
    import { Point } from "$data/geometry";
    import { pop } from "$data/iterators";
    import { automateContext } from "$data/automate/automate";
    import { completeConnection, type SlotRef } from '$data/automate/node';

    export let id: SlotRef;
    export let kind: ValueKind;
    export let direction: "input" | "output";
    export let multiple: boolean = false;

    const { layout, anchors, blockPan, startedConnection, connections } = automateContext();

    const nodeRect = layout.get(id.nodeId);

    let self: HTMLDivElement;
    let canReceive: boolean = true;

    const anchor = anchors(id);

    function mouseDown() {
        blockPan.set(true);

        if (direction === "input" && !multiple) {
            // We we are an input and can't receive multiple values
            // We detach the current connection instead if we have one

            const conn = pop(connections.get(id));

            if (conn) {
                connections.remove(conn);

                startedConnection.set({
                    start: conn.from,
                    startDirection: "output",
                    kind,
                    over: id,
                });
                return;
            }
        }

        startedConnection.set({
            start: id,
            startDirection: direction,
            kind,
        });      
    }

    function mouseUp() {
        if ($startedConnection?.over === id) {
            connections.add(completeConnection($startedConnection, id));
        }
    }

    function mouseEnter() {
        if (!canReceive) return;
        startedConnection.update(hf => {
            if (!hf) return null;
            return {
                ...hf,
                over: id,
            }
        });
    }

    function mouseLeave() {
        if (!canReceive) return;
        startedConnection.update(hf => {
            if (!hf || !hf.over?.same(id)) return hf;
            return {
                ...hf,
                over: null,
            }
        });
    }

    $: color = kind.type === 'any' ? 
        'linear-gradient(328deg, rgba(140,108,255,1) 10%, rgba(255,108,109,1) 90%)' : 
        colors[kind.type].toString();

    // Update this slots anchor
    $: if (self) {
        let offset_x = 6;
        let offset_y = 6;

        if (multiple) {
            offset_y = 18;
        }

        anchor.set(
            new Point(
                $nodeRect.origin.x + self.offsetLeft + offset_x,
                $nodeRect.origin.y + self.offsetTop + offset_y
            )
        );
    }

    $: if ($startedConnection) {
        canReceive = (
            ($startedConnection.kind.type === kind.type || $startedConnection.kind.type === 'any' || kind.type === 'any') && // Same kind
            ($startedConnection.start.nodeId !== id.nodeId) && // Not on the same node
            ($startedConnection.startDirection !== direction) && // Opposite directions
            (direction === 'output' || multiple || !pop(connections.get(id))) // Not full
        );

        if (canReceive) {
            // We have made the easy tests, now check for uniqueness
            canReceive = !connections.connected($startedConnection.start, id);
        }
    } else {
        canReceive = true;
    }
</script>

<div class="sensor" 
    class:multiple
    on:mouseenter={mouseEnter}
    on:mouseleave={mouseLeave}
    on:mousedown={mouseDown}
    on:mouseup={mouseUp}
    style="--kind-color: {color}"
    >
    <div 
        class="anchor" 
        class:incompatible={!canReceive}
        class:construction={$startedConnection && $startedConnection.start.same(id)}
        bind:this={self}>
    </div>
</div>


<style>
    .sensor {
        height: 30px;
        width: 30px;
        border-radius: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .sensor.multiple {
        height: 48px;
    }
 
    .anchor {
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;
        border-radius: 6px;
        background: var(--kind-color);

        box-shadow: inset 0 0 1px 2px rgba(255, 255, 255, 0.45); 
        
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center ;
    }

    .sensor:hover .anchor:not(.incompatible), .anchor.construction {
        box-shadow: inset 0 0 0 3px rgba(255, 255, 255, 0.8); 
    }

    .sensor.multiple .anchor {
        height: 36px;
    }

    .anchor.incompatible:not(.construction) {
        background-color: var(--icon);
    } 
</style>