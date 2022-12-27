<script lang="ts">
    import { automateContext } from "$data/automate/automate";
    import colors from "$data/colors";
    import Icon from "$lib/Icon.svelte";
    import { createEventDispatcher } from "svelte";
    import { get } from "svelte/store";

    export let zoom: number;

    const dispatch = createEventDispatcher();
    const { layout, selected } = automateContext();

    function alignTop() {
        const sel = get(selected);
        const layouts = Array.from(sel)
            .map((n) => layout.get(n))
            .sort((a, b) => get(a).origin.y - get(b).origin.y);

        if (layouts.length === 0) {
            return;
        }

        let base = get(layouts[0]).origin.y;
        base = Math.floor((base - 0.1) / 20) * 20;

        for (const layout of layouts) {
            layout.moveY(base);
        }
    }

    function home() {
        dispatch("home", null);
    }
</script>

<div class="top-menu">
    <ul class="icon-row">
        <li>
            Save
            <Icon name="device-floppy" color={colors.fadedtext} size={16} />
        </li>
    </ul>

    <div class="right-group">
        <ul class="icon-row">
            <li title="Convert automation to Value Buffer">
                <Icon name="logic-buffer" color={colors.fadedtext} size={18} />
            </li>
        </ul>

        <ul class="icon-row">
            <li title="Align top" on:click={alignTop}>
                <Icon name="box-align-top" color={colors.fadedtext} size={18} />
            </li>
            <li title="Align bottom">
                <Icon
                    name="box-align-bottom"
                    color={colors.fadedtext}
                    size={18}
                />
            </li>
            <li title="Align left">
                <Icon
                    name="box-align-left"
                    color={colors.fadedtext}
                    size={18}
                />
            </li>
            <li title="Align right">
                <Icon
                    name="box-align-right"
                    color={colors.fadedtext}
                    size={18}
                />
            </li>
        </ul>

        <ul class="icon-row">
            <li on:click={home} title="Home view">
                <Icon name="home" color={colors.fadedtext} size={18} />
            </li>
        </ul>

        <div class="zoom-box">
            <div class="plus" title="Zoom in" on:click={() => (zoom += 0.1)}>
                +
            </div>
            <div class="value">{(zoom * 100).toFixed(0)}%</div>
            <div class="minus" title="Zoom out" on:click={() => (zoom -= 0.1)}>
                -
            </div>
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

    .zoom-box > div {
        padding: 3px;
        transition: 200ms background-color;
    }

    .zoom-box > .value {
        width: 40px;
        text-align: center;
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

    .icon-row > li {
        display: flex;
        justify-content: center;
        align-items: center;
        background-color: var(--container);
        padding: 0 9px;
        gap: 6px;
    }

    .icon-row > li:hover {
        background-color: var(--containerhigh);
    }

    .icon-row > li:active {
        background-color: var(--container);
    }

    .icon-row > li:first-child {
        border-top-left-radius: 4px;
        border-bottom-left-radius: 4px;
    }

    .icon-row > li:last-child {
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
