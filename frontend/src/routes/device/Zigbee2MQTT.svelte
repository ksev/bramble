<script lang="ts">
    import Colors from '$data/colors';
    import Ledger from '$lib/Ledger.svelte';
    import Icon from '$lib/Icon.svelte';

    import Section from '$lib/Section.svelte';
    import SubMenu from '$lib/SubMenu.svelte';
    import TextInput from '$lib/TextInput.svelte';
    import api from '$data/api';
    import { devices } from '$data/devices';
    import { error, success } from '$data/notification';
    import { derived } from 'svelte/store';
    import { count } from '$data/iterators';

    let host: string;
    let password: string;
    let username: string;
    let port = 1883;

    let connected = devices.filtered(d => d.deviceType.name === "zigbee2mqtt");

    let isEmpty = true;
    $: isEmpty = $connected.length === 0;

    async function connect() {
        try {
            await $api.addZigbee2MqttIntegration({
                host,
                port,
                username,
                password
            });

            success("CONNECTED");
        } catch (e) {
            error(e.toString());
        } 
    }
</script>

<h1>Zigbee2MQTT</h1>

<p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam dapibus fermentum nulla nec fringilla. Nulla finibus ligula eu purus consectetur posuere. Etiam ac vulputate libero. In porta elit non ante eleifend, eget convallis libero porta. Nulla facilisi. Nulla sed sem urna. Praesent ipsum nunc, tincidunt eu tristique quis, rutrum ut purus. Ut dapibus porttitor lacinia. 
</p>
<p>
    Quisque convallis maximus efficitur. Donec at elit augue. Integer facilisis libero ac erat bibendum iaculis. Pellentesque interdum semper eros non euismod. Nulla molestie nibh non turpis faucibus, in porta ligula auctor. Vestibulum quam lorem, feugiat mattis urna et, laoreet semper eros. Nullam rhoncus tristique dictum. Vestibulum placerat varius turpis ac sagittis. 
</p>

<div class="ledger">
    <Ledger>
        <div>
            <Icon name="cpu" color={Colors.device} size={18} />
            <span>Device</span>
        </div>
    </Ledger>
</div>

<Section color={Colors.automation}>
    <span slot="headline">Connected servers</span>
    <div slot="content" class:empty={isEmpty}>
        {#if isEmpty}
            <span>
                Not connected to <strong>Zigbee2MQTT</strong>
            </span>
        {:else}
            {#each $connected as server}
                <div class="server">
                    <div class="host">{server.name}</div>
                    <div class="devices"><Icon name="cpu" color={Colors.device} size={18} /> {count(devices.children(server.id))}</div>
                </div>
            {/each} 
        {/if}
    </div>
</Section>

<Section color={Colors.device}>
    <span slot="headline">Connect to Zigbee2MQTT</span>
    <div slot="content">
        <section class="form">
            <TextInput label="MQTT server address" bind:value={host} />
            <TextInput label="Username" bind:value={username} />
            <TextInput label="Password" bind:value={password} />
        </section>
        
        <section class="menu">
            <SubMenu>
                <button on:click={connect} disabled={!$api}>
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

    .ledger {
        max-width: 1000px;
        display: flex;
        justify-content: end;
        margin-top: 24px;
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

    .server {
        display: flex;
        align-items: stretch;
        margin-top: 3px;
        gap: 3px;
    }

    .server > * {
        border-radius: 2px;
        background-color: var(--container);
        white-space: nowrap;
        padding: 8px;
    }

    .server .host {
        flex-grow: 1;
    }
    
    .server .devices {
        display: flex;
        gap: 3px;
        align-items: center;
    }   

</style>