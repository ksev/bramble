<script lang="ts">    
    import Zigbee2Mqtt from '../../assets/zigbee2mqtt.svg';
    import Mqtt from '../../assets/mqtt.svg';

    import Section from '$lib/Section.svelte';
    import Icon from '$lib/Icon.svelte';
    import SubMenu from '$lib/SubMenu.svelte';

    import Colors from '$data/colors';

    const virtualGradient = `linear-gradient(180deg, ${Colors.feature} 50%, ${Colors.automation} 90%)`;
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
                        <!--
                        <div class="integration-summary">
                            <div class="stat">
                                <Icon name="microchip" size={16} color={Colors.device} />
                                <strong>1</strong> 
                                Devices
                            </div>
                            <div class="stat">
                                <Icon name="activity" size={16} color={Colors.source} />
                                <strong>5</strong> 
                                Sources
                            </div>
                            <div class="stat">
                                <Icon name="archive" size={16} color={Colors.sink} />
                                <strong>2</strong> 
                                Sinks
                            </div>
                        </div>
                        -->
                        <img src={Mqtt} alt="Zigbee2MQTT" />
                    </a>

                    <a class="integration" href="#/device/add/virtual">
                        <span>Virtual <strong>Device</strong></span>
                        <Icon name="ghost" color={virtualGradient} size={75} />
                    </a>
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

    /*

    .integration-summary {
        display: flex;
        flex-direction: column; 
        gap: 2px;
        flex-grow: 1;
    }

    .integration .stat {
        display: flex;
        gap: 6px;
        align-items: center;
    }

    */

    .wrap {
        margin-top: 24px;
        display: flex;
    }
</style>