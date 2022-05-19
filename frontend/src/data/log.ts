export function info(message: string) {
    console.log('[info]', message);
}

export function warn(message: string) {
    console.warn('[warn]', message);
}

export function debug(message: string) {
    console.debug('[debug]', message);
}

export function error(message: string) {
    console.error('[error]', message);
}

export function trace(message: string) {
    console.trace('[trace]', message);
}