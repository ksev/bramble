import Home from './routes/Home.svelte';
import Devices from './routes/Devices.svelte';
import DeviceAdd from './routes/device/Add.svelte';
import NotFound from './routes/NotFound.svelte';

export default {
    '/': Home,
    '/devices': Devices,
    '/device/add': DeviceAdd,
    // The catch-all route must always be last
    '*': NotFound
};