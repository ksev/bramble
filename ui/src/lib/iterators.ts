export function* monotonic() {
    for(let i = 0;;i++) {
        yield i;
    }
}

export function* map<T, O>(it: Iterable<T>, callback: (data: T, index?: number) => O) {
    let index = 0;

    for (const v of it) {
        yield callback(v, index++);
    }
}

export function* flatMap<T, O>(it: Iterable<T>, callback: (data: T, index?: number) => Iterable<O>) {
    let index = 0;

    for (const v of it) {
        const it2 = callback(v, index++);
        for (const v2 of it2) {
            yield v2;
        }
    }
}

export function* take<T>(it: IterableIterator<T>, n: number) {
    for (let i = 0; i < n; i++) {
        const r = it.next();

        if (r.done === true) {
            return;
        }

        yield r.value;
    }
}

export function next<T>(value: IterableIterator<T>): T | null {
    const n = value.next();

    if (n.done === true) {
        return null;
    }
    
    return n.value;
}   

export function* repeat<T>(value: T, n: number) {
    for(let i = 0; i < n; i++) {
        yield value;
    }
}

export function* chain<T>(...args: Iterable<T>[]){
    for(const it of args) {
        for(const v of it) {
            yield v;
        }
    }
}

export function* window<T>(it: IterableIterator<T>, size: number) {
    let stack = Array.from(take(it, 4));

    // Yield first window
    yield stack;

    // Walk the rest of the iterator
    for (const v of it) {
        stack = [...stack.slice(-(size - 1)), v];
        yield stack;
    }
}

export type CatmullOutput = [[number, number], [number, number], [number, number]];

export function* catmull(it: IterableIterator<[number, number]>, length: number, k: number) {
    if (length < 2) {
        return;
    }

    const first = next(it);

    const points = chain(
        repeat(first, 2),
        take(it, length - 2),        
        flatMap(it, (v) => repeat(v, 2))   
    );

    yield [first, [0,0], [0,0]] as CatmullOutput;

    for (const [[x0, y0], [x1, y1], [x2, y2], [x3, y3]] of window(points, 4)) {            
        var cp1x = x1 + (x2 - x0) / 6 * k;
        var cp1y = y1 + (y2 - y0) / 6 * k;

        var cp2x = x2 - (x3 - x1) / 6 * k;
        var cp2y = y2 - (y3 - y1) / 6 * k;
    
        yield [[x2, y2], [cp1x, cp1y], [cp2x, cp2y]] as CatmullOutput;
    }
} 
