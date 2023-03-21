<script lang="ts">    
    import Zigbee2Mqtt from '../../assets/zigbee2mqtt.svg';
    import Mqtt from '../../assets/mqtt.svg';
    import { devices } from '$data/devices';

    import Section from '$lib/Section.svelte';
    import Icon from '$lib/Icon.svelte';
    import SubMenu from '$lib/SubMenu.svelte';

    import Colors from '$data/colors';
    import IntegrationSummary from '$lib/IntegrationSummary.svelte';
    import Api from '$data/api';

    const virtualGradient = `linear-gradient(180deg, ${Colors.feature} 50%, ${Colors.automation} 90%)`;

    let sun = devices.single("thesun");

    function setSunLocation() {
        navigator.geolocation.getCurrentPosition(p => {
            $Api.setSunLocation({
                lat: p.coords.latitude,
                lon: p.coords.longitude,
            })
        });
    }
</script>

<div>
    <h1>Add Device</h1>

    <p>
        Lorem ipsum dolor <strong>Device</strong> sit amet, consectetur adipiscing elit. 
        Sed purus nibh, dictum sit amet urna id, bibendum dapibus metus.
        Pellentesque viverra a est id consequat. 
    </p>

    <Section color={Colors.device}>
        <span slot="headline">Device</span>

        <div slot="content">
            <p>
                Lorem ipsum dolor <strong>Direct</strong> sit amet, consectetur adipiscing elit. 
                Sed purus nibh, dictum sit amet urna id, bibendum dapibus metus.
                Pellentesque viverra a est id consequat. 
            </p>

            <div class="wrap">
                <SubMenu>
                    <a class="integration" href="#/device/add/mqtt">
                        <span>MQTT</span>
                        <img src={Mqtt} alt="Zigbee2MQTT" />
                    </a>

                    <a class="integration" href="#/device/add/virtual">
                        <span>Virtual <strong>Device</strong></span>
                        <Icon name="ghost" color={virtualGradient} size={75} />
                    </a>
                    
                    <button class="integration" on:click={setSunLocation}>
                        <span>The <strong>Sun</strong></span>
                        <Icon name="sunset-2"  color={Colors.source} size={75} />

                        {#if $sun} 
                            <div class="sun-loc">
                                <div>
                                    <Icon name="map-pin" color={Colors.source} size={18} />
                                </div>
                                <div>
                                    (active)
                                </div>
                            </div>
                        {/if}
                    </button>
                </SubMenu>
            </div>
        </div>
    </Section>

    <Section color={Colors.device}>
        <span slot="headline">Integration</span>
        <div slot="content">
            <p>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                Sed purus nibh, <strong>Integration</strong> dictum sit amet urna id, bibendum dapibus metus.
                Pellentesque viverra a est id consequat. 
            </p>

            <div class="wrap">
                <SubMenu>
                    <a class="integration" href="#/device/add/zigbee2mqtt">
                        <span>Zigbee2MQTT</span>
                        <img src={Zigbee2Mqtt} alt="Zigbee2MQTT" />
                        <IntegrationSummary name="zigbee2mqtt" />
                    </a>
                </SubMenu>
            </div>
        </div>
    </Section>
</div>

<style>
    .integration {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: 24px;
        gap: 12px;
        transition: width 500ms linear;
    }

    .integration img {
        width: 75px;
        height: 75px;
    }

    .integration span {
        text-align: center;
    }

    .wrap {
        margin-top: 24px;
        display: flex;
        flex-wrap:wrap;
    }

    .sun-loc {
        display: flex;
        flex-direction: column;
        gap: 6px;
        align-items: center;
    }
</style>