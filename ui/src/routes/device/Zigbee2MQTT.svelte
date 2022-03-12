<script lang="ts">
    import Colors from '$data/colors';

    import Section from '$lib/Section.svelte';
    import SubMenu from '$lib/SubMenu.svelte';
    import type { Readable } from 'svelte/store';
    import { configService } from '$net/pipe';
    import type { ConfigResult } from '$net/protocol';

    let url: string;
    let password: string;
    let username: string;

    let configResult: Readable<ConfigResult>;

    function changeSettings(){
        configResult = $configService.zigbee2MQTT({
            url,
            username,
            password
        });
    }

    $: console.log($configResult);
</script>

<h1>Zigbee2MQTT</h1>

<p>
    Lorem ipsum dolor sit, consectetur adipiscing elit. Pellentesque pulvinar elit sed quam ultrices mattis. Sed at convallis quam. In. 
</p>

<Section color={Colors.sink}>
    <span slot="headline">Status</span>
    <div slot="content" class="empty">
        <span>
            Not connected to <strong>Zigbee2MQTT</strong>
        </span>
    </div>
</Section>

<Section color={Colors.source}>
    <span slot="headline">Settings</span>
    <div slot="content">
        <section class="form">
            <div class="form-group">
                <label for="input">MQTT server address</label>
                <input id="input" bind:value={url} />
            </div>
        
            <div class="form-group">
                <label for="input">Username</label>
                <input id="input" bind:value={username} />
            </div>
        
            <div class="form-group">
                <label for="input">Password</label>
                <input id="input" bind:value={password} />
            </div>
        </section>
        
        <section class="menu">
            <SubMenu>
                <button on:click={changeSettings} disabled={!$configService}>
                    Connect
                </button>
            </SubMenu>
        </section>
    </div>
</Section>

<style>
    .form {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .menu {
        display: flex;
        justify-content: right;
    }
  
    .form-group {
        display: flex;
        overflow: hidden;
        
        padding: 0;
        gap: 3px;
    }

    label {
        background-color: var(--source);
        padding: 8px;
        display: flex;
        align-items: center;
        border-radius: 2px;
        color: var(--strong);
    }

    input {
        background-color: var(--container);
        border: none;
        outline: none;
        padding: 8px;
        flex-grow: 1;
        color: #fafafa;
        
        border-radius: 2px;
    }

    .empty {
        background-color: var(--container);
        border-radius: 2px;
        margin-top: 3px;
        text-align: center;
        padding: 24px;
        display: flex;
        line-height: 24px;
        justify-content: center;
        align-items: center;
        color: var(--fadedtext);
    }
</style>