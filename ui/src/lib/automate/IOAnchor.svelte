<script lang="ts">
    import { Point } from "$data/geometry";
    import { automateContext, completeEdge, IOId, IOType } from "./automate";

    export let id: IOId;
    export let type: IOType;
    export let direction: "input" | "output";

    const { layout, anchors, blockPan, halfEdge, edges } = automateContext();

    const rect = layout.get(id.nodeId);

    let anchor: HTMLDivElement;
    let disabled: boolean;

    function mouseDown() {
        blockPan.set(true);

        if (direction === "output") {
            halfEdge.set({
                output: id,
                type,
            });
        } else {
            halfEdge.set({
                input: id,
                type,
            });
        }        
    }

    function mouseUp() {
        if (disabled) return;
        edges.update(edges => {
            return [
                ...edges,
                completeEdge($halfEdge, id),
            ]
        });
    }

    function mouseEnter() {
        if (disabled) return;

        halfEdge.update(hf => {
            if (!hf) return null;
            return {
                ...hf,
                over: id,
            }
        });
    }

    function mouseLeave() {
        if (disabled) return;

        halfEdge.update(hf => {
            if (!hf || hf.over !== id) return null;
            return {
                ...hf,
                over: null,
            }
        });
    }

    $: if (anchor) {
        anchors(id).set(
            new Point(
                $rect.origin.x + anchor.offsetLeft + 6,
                $rect.origin.y + anchor.offsetTop + 6
            )
        );
    }

    $: if ($halfEdge) {
        disabled = (
            direction in $halfEdge || 
            type.kind !== $halfEdge.type.kind           
        );
    } else {
        disabled = false;
    }
</script>

<div 
    class="icon numeric" 
    class:disabled={disabled && $halfEdge[direction] !== id}
    bind:this={anchor} 
    on:mouseenter={mouseEnter}
    on:mouseleave={mouseLeave}
    on:mousedown={mouseDown}
    on:mouseup={mouseUp}>
</div>

<style>
    .icon {
        background-color: var(--device);
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;

        border: 3px solid var(--device); 

        transition: 100ms linear box-shadow, 300ms linear filter;        
    }

    .icon:hover:not(.disabled) {
        box-shadow: 0 0 4px rgba(0,0,0,0.35) inset;
    }

    .icon.numeric {
        border-radius: 6px;
    }

    .icon.disabled {
        filter: saturate(0);
    }
</style>