import { readable } from "svelte/store";

export function keyPressed(key: string) {
    return readable(false, (set) => {
        const keydown = (e: KeyboardEvent) => {
            if (e.key === key) set(true);
        };

        const keyup = (e: KeyboardEvent) => {
            if (e.key === key) set(false);
        }

        window.addEventListener('keydown', keydown);
        window.addEventListener('keyup', keyup);

        return () => {
            window.removeEventListener('keyup', keyup);
            window.removeEventListener('keydown', keydown);
        };
    });
}