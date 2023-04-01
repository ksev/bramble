<script lang="ts">
    import { automateContext } from "$data/automate/automate";
    import colors from "$data/colors";
    import { Point, Rect } from "$data/geometry";
    import Icon from "$lib/Icon.svelte";
    import { createEventDispatcher } from "svelte";
    import { get } from "svelte/store";

    export let zoom: number;

    const dispatch = createEventDispatcher();
    const { layout, selected } = automateContext();

    function alignTop() {
        // Get the rects for the selected nodeboxes and sort them by +y
        // So the node that is the furthest up is first
        const layouts = Array.from($selected)
            .map(n => [n, get(layout.get(n))] as [number, Rect])
            .sort(([_i, a], [_ii, b]) => a.origin.y - b.origin.y);

        if (layouts.length === 0) {
            return;
        }

        let [_, first] = layouts[0];
        const base = Math.floor(first.origin.y / 20) * 20;

        for (let i = 0; i < layouts.length; i++) {
            const [id, rect] = layouts[i];        

            // Create a rect for the space between where we are an where we want to go
            const moveSpace = Rect.corners(
                new Point(rect.origin.x, base),
                new Point(rect.br().x, rect.origin.y),
            );

            const hits = layouts.filter(([_, r]) => r.intersect(moveSpace))
                                .sort(([_i, a], [_ii, b]) => b.origin.y - a.origin.y);

            let horizon = base;

            if (hits.length > 0) {
                horizon = hits[0][1].bl().y + 20;
            }

            layouts[i] = [id, rect.moveTo(new Point(rect.origin.x, horizon))];
        }


        for (const [id, newRect] of layouts) {
            layout.get(id).moveY(newRect.origin.y);   
        }
    }

    function alignBottom() {
        // Get the rects for the selected nodeboxes and sort them by +y
        // So the node that is the furthest up is first
        const layouts = Array.from($selected)
            .map(n => [n, get(layout.get(n))] as [number, Rect])
            .sort(([_i, a], [_ii, b]) => b.bl().y - a.bl().y);

        if (layouts.length === 0) {
            return;
        }

        let [_, first] = layouts[0];
        const base = Math.ceil(first.bl().y / 20) * 20;

        for (let i = 0; i < layouts.length; i++) {
            const [id, rect] = layouts[i];        

            // Create a rect for the space between where we are an where we want to go
            const moveSpace = Rect.corners(
                rect.bl(),
                new Point(rect.br().x, base),
            );

            const hits = layouts.filter(([_, r]) => r.intersect(moveSpace))
                                .sort(([_i, a], [_ii, b]) => a.bl().y - b.bl().y);

            let horizon = base;

            if (hits.length > 0) {
                horizon = hits[0][1].origin.y - 20;
            }

            layouts[i] = [id, rect.moveTo(new Point(rect.origin.x, horizon - rect.size.height))];
        }


        for (const [id, newRect] of layouts) {
            layout.get(id).moveY(newRect.origin.y);   
        }
    }
    
    function alignLeft() {
        // Get the rects for the selected nodeboxes and sort them by +y
        // So the node that is the furthest up is first
        const layouts = Array.from($selected)
            .map(n => [n, get(layout.get(n))] as [number, Rect])
            .sort(([_i, a], [_ii, b]) => a.origin.x - b.origin.x);

        if (layouts.length === 0) {
            return;
        }

        let [_, first] = layouts[0];
        const base = Math.floor(first.origin.x / 20) * 20;

        for (let i = 0; i < layouts.length; i++) {
            const [id, rect] = layouts[i];        

            // Create a rect for the space between where we are an where we want to go
            const moveSpace = Rect.corners(
                rect.tl(),
                new Point(base, rect.bl().y),
            );

            const hits = layouts.filter(([_, r]) => r.intersect(moveSpace))
                                .sort(([_i, a], [_ii, b]) => b.tr().x - a.tr().x);

            let horizon = base;

            if (hits.length > 0) {
                horizon = hits[0][1].tr().x + 20;
            }

            layouts[i] = [id, rect.moveTo(new Point(horizon, rect.origin.y))];
        }

        for (const [id, newRect] of layouts) {
            layout.get(id).moveX(newRect.origin.x);   
        }
    }

    function alignRight() {
        // Get the rects for the selected nodeboxes and sort them by +y
        // So the node that is the furthest up is first
        const layouts = Array.from($selected)
            .map(n => [n, get(layout.get(n))] as [number, Rect])
            .sort(([_i, a], [_ii, b]) => b.tr().x - a.tr().x);

        if (layouts.length === 0) {
            return;
        }

        let [_, first] = layouts[0];
        const base = Math.ceil(first.tr().x / 20) * 20;

        for (let i = 0; i < layouts.length; i++) {
            const [id, rect] = layouts[i];        

            // Create a rect for the space between where we are an where we want to go
            const moveSpace = Rect.corners(
                rect.tr(),
                new Point(base, rect.br().y),
            );

            const hits = layouts.filter(([_, r]) => r.intersect(moveSpace))
                                .sort(([_i, a], [_ii, b]) => a.tr().x - b.tr().x);

            let horizon = base;

            if (hits.length > 0) {
                horizon = hits[0][1].origin.x - 20;
                
            }

            layouts[i] = [id, rect.moveTo(new Point(horizon - rect.size.width, rect.origin.y))];
        }


        for (const [id, newRect] of layouts) {
            layout.get(id).moveX(newRect.origin.x);   
        }
    }

    function home() {
        dispatch("home", null);
    }

    function save() {
        dispatch("save", null);    
    }
</script>

<div class="top-menu">
    <div class="icon-row">
        <button on:click={save}>
            Save
            <Icon name="device-floppy" color={colors.fadedtext} size={16} />
        </button>
    </div>

    <div class="right-group">
        <div class="icon-row">
            <button title="Convert automation to Value Buffer">
                <Icon name="logic-buffer" color={colors.fadedtext} size={18} />
            </button>
        </div>

        <div class="icon-row">
            <button title="Align top" on:click|stopPropagation={alignTop}>
                <Icon name="box-align-top" color={colors.fadedtext} size={18} />
            </button>
            <button title="Align bottom" on:click|stopPropagation={alignBottom}>
                <Icon
                    name="box-align-bottom"
                    color={colors.fadedtext}
                    size={18}
                />
            </button>
            <button title="Align left" on:click|stopPropagation={alignLeft}>
                <Icon
                    name="box-align-left"
                    color={colors.fadedtext}
                    size={18}
                />
            </button>
            <button title="Align right" on:click|stopPropagation={alignRight}>
                <Icon
                    name="box-align-right"
                    color={colors.fadedtext}
                    size={18}
                />
            </button>
        </div>

        <div class="icon-row">
            <button on:click={home} title="Home view">
                <Icon name="home" color={colors.fadedtext} size={18} />
            </button>
        </div>

        <div class="zoom-box">
            <button class="plus" title="Zoom in" on:click={() => (zoom += 0.1)}>
                +
            </button>
            <div class="value">{(zoom * 100).toFixed(0)}%</div>
            <button class="minus" title="Zoom out" on:click={() => (zoom -= 0.1)}>
                -
            </button>
        </div>
    </div>
</div>

<style>
    .top-menu {
        z-index: 950;
        padding: 12px;
        background-color: rgba(31, 31, 51, 0.8);
        color: var(--fadedtext);
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 51px;
    }

    .zoom-box {
        display: inline-flex;
        flex-direction: row;
        gap: 3px;
        background-color: var(--container);
        border-radius: 4px;
        overflow: hidden;
        justify-content: center;
        align-items: center;
        padding: 2px;
        height: 27px;
    }

    .zoom-box > * {
        padding: 3px;
        transition: 200ms background-color;
    }

    .zoom-box > .value {
        width: 40px;
        text-align: center;
    }

    .zoom-box > button {
        background-color: var(--container);
        display: flex;    
        align-items: center;
        justify-content: center;
    }

    .zoom-box > .plus,
    .zoom-box > .minus {
        width: 20px;
        text-align: center;
        border-radius: 4px;
    }

    .zoom-box > .plus:hover,
    .zoom-box > .minus:hover {
        background-color: var(--background);
    }

    .zoom-box > .plus:active,
    .zoom-box > .minus:active {
        background-color: var(--container);
    }

    .icon-row {
        display: inline-flex;
        gap: 1px;
        height: 27px;
    }

    .icon-row > button {
        display: flex;
        justify-content: center;
        align-items: center;
        background-color: var(--container);
        padding: 0 9px;
        gap: 6px;
    }

    .icon-row > button:hover {
        background-color: var(--containerhigh);
    }

    .icon-row > button:active {
        background-color: var(--container);
    }

    .icon-row > button:first-child {
        border-top-left-radius: 4px;
        border-bottom-left-radius: 4px;
    }

    .icon-row > button:last-child {
        border-top-right-radius: 4px;
        border-bottom-right-radius: 4px;
    }

    .right-group {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 12px;
    }
</style>
