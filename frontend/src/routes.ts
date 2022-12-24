import Home from './routes/Home.svelte';
import Devices from './routes/Devices.svelte';
import DeviceAdd from './routes/device/Add.svelte';
import Virtual from './routes/device/Virtual.svelte';
import Zigbee2MQTT from './routes/device/Zigbee2MQTT.svelte';
import Automate from './routes/device/Automate.svelte';
import NotFound from './routes/NotFound.svelte';

export default {
    '/': Home,
    '/devices': Devices,
    '/device/add': DeviceAdd,
    '/device/add/zigbee2mqtt': Zigbee2MQTT,
    '/device/add/virtual': Virtual,
    '/device/:deviceid/:property/automate': Automate,
    // The catch-all route must always be last
    '*': NotFound
};