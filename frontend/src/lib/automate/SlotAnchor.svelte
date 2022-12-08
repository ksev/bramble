<script lang="ts">
    import type { ValueKind } from "$data/device";
    import { Point } from "$data/geometry";
  import { pop } from "$data/iterators";
  import Icon from "$lib/Icon.svelte";
    import { automateContext, completeConnection, type SlotRef } from './automate';

    export let id: SlotRef;
    export let kind: ValueKind;
    export let direction: "input" | "output";
    export let multiple: boolean = false;

    const { layout, anchors, blockPan, startedConnection, connections } = automateContext();

    const rect = layout.get(id.nodeId);

    let anchor: HTMLDivElement;
    let canReceive: boolean = true;

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
        if (!canReceive && $startedConnection) return;
        connections.add(completeConnection($startedConnection, id));
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

    $: if (anchor) {
        let offset_x = 6;
        let offset_y = 6;

        if (multiple) {
            offset_y = 18;
        }

        anchors(id).set(
            new Point(
                $rect.origin.x + anchor.offsetLeft + offset_x,
                $rect.origin.y + anchor.offsetTop + offset_y
            )
        );
    }

    $: if ($startedConnection) {
        const cons = Array.from(connections.get(id));
        const sameKind = $startedConnection.kind.type === kind.type;
        const oppositeDirection = $startedConnection.startDirection !== direction;
        const notFull = direction === 'output' || (multiple || cons.length === 0);
        const notSameNode = $startedConnection.start.nodeId !== id.nodeId;

        canReceive = (
            sameKind &&
            oppositeDirection &&
            notFull &&
            notSameNode
        );

        if (canReceive) {
            // We have made the easy tests, now check for uniqueness
            canReceive = !connections.has({
                from: $startedConnection.start,
                to: id,
                kind,
            });
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
    on:mouseup={mouseUp}>
    <div 
        class="icon" 
        class:incompatible={!canReceive}
        class:construction={$startedConnection && $startedConnection.start.same(id)}
        bind:this={anchor}>
    </div>
</div>


<style>
    .sensor {
        height: 24px;
        width: 24px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .sensor.multiple {
        height: 48px;
    }
 
    .icon {
        background-color: var(--device);
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;
        border-radius: 6px;

        border: 2px solid rgba(255,255,255,0.2); 

        transition: 100ms linear box-shadow, 300ms linear filter, 500ms background-color, 500ms border;        
    }

    .sensor:hover .icon:not(.incompatible), .icon.construction {
        box-shadow: 0 0 3px rgba(0,0,0,0.2) inset;
    }

    .sensor.multiple .icon {
        height: 36px;
    }

    .icon.incompatible:not(.construction) {
        background-color: var(--icon);
    }
</style>