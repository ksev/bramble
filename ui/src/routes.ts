import Home from './routes/Home.svelte';
import Devices from './routes/Devices.svelte';
import DeviceAdd from './routes/device/Add.svelte';
import Zigbee2MQTT from './routes/device/Zigbee2MQTT.svelte';
import NotFound from './routes/NotFound.svelte';

export default {
    '/': Home,
    '/devices': Devices,
    '/device/add': DeviceAdd,
    '/device/add/zigbee2mqtt': Zigbee2MQTT,
    // The catch-all route must always be last
    '*': NotFound
};