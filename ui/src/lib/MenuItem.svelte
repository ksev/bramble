<script lang="ts">
    import Icon from "./Icon.svelte";
    import {location} from 'svelte-spa-router'
    import { colors } from '../colors';
    
    export let url: string;
    export let icon: string;

    let active = false;
    $: active = $location === url;
</script>

<li class:active>
    <a href={`#${url}`}><Icon name={icon} color={ active ? colors.source : colors.icon } /></a>
</li>

<style>
    li {    
        width: 48px;
        height: 48px;
        border-radius: 15px;
        margin-top: 18px;
        transition: background-color 200ms linear;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
    }

    li.active {
        background-color: var(--container);
    }
</style>