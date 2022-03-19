<script lang="ts">
import { Point } from "$data/geometry";

    import { automateContext, Input } from "./automate";

    export let nodeId: number;
    export let data: Input;

    const { layout, anchors, blockPan } = automateContext();
    const rect = layout.get(nodeId);

    let anchor: HTMLDivElement;

    $: if (anchor) {
        const key: [number, string] = [nodeId, data.id];
        anchors(key).set(
            new Point(
                $rect.origin.x + anchor.offsetLeft + 6,
                $rect.origin.y + anchor.offsetTop + 6
            )
        );
    }
</script>

<div class="input">
    <div class="icon numeric" bind:this={anchor}></div>            
    {data.label}        
    <!--
    <div class="input-cont">
        <input />
    </div>  
    -->
</div>

<style>
    .icon {
        background-color: var(--device);
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;

        border: 3px solid var(--device);
        transition: 200ms linear box-shadow;
    }

    .icon.numeric {
        border-radius: 6px;
    }

    .icon:hover, .icon.connected {
        box-shadow: 0 0 4px rgba(0,0,0,0.35) inset;
    }

    .input {        
        height: 20px;
        display: flex;
        justify-content: left;
        align-items: center;
        gap: 8px;

        margin-left: -6px;
    }

    /*
    .input .input-cont {
        padding-right: 8px;
        flex-shrink: 1;
    }

    .input input {
        border: none;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 4px;
        background-color: var(--container);
        border-radius: 4px;
        font-size: 12px;
    }
    */
</style>