import Home from './routes/Home.svelte';
import Devices from './routes/Devices.svelte';
import NotFound from './routes/NotFound.svelte';

export default {
    '/': Home,
    '/devices': Devices,
    // The catch-all route must always be last
    '*': NotFound
};