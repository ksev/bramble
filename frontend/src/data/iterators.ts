/**
 * A generator that generates a monotonic squence of numbers starting with 0
 */
export function* monotonic() {
    for(let i = 0;;i++) {
        yield i;
    }
}

/**
 * Spit out random number between 0->1 for eternity
 */
export function* random() {
    for (;;) {
        yield Math.random();
    }
}

export function* randomCharacters() {
    for (let r of random()) {
        yield Math.random().toString(36).replace('0.', '');  
    }
}

/**
 * Gives an exponentially larger number every iteration to the max
 * @param start The starting number and base number for exponential growth
 * @param max The max number allowed, defaults to MAX_SAFE_INTEGER
 */
export function* exponential(start: number, max: number = Number.MAX_SAFE_INTEGER) {
    for (let i = 0;; i++) {
        const num = start * Math.pow(2, i);

        if (isNaN(num) || num > max) {
            yield max;
            continue;
        }

        yield num;
    }
} 

/**
 * Map over an Iterators values to transform them
 * @param it Iterator 
 * @param callback Function to transform the value in the iterator
 */
export function* map<T, O>(it: Iterable<T>, callback: (data: T, index?: number) => O) {
    if (!it) return;

    let index = 0;

    for (const v of it) {
        yield callback(v, index++);
    }
}

/**
 * Aync version of Map, Map over an Async iterator 
 * @param it Iterator
 * @param callback Async callback that generates new values
 * @returns An iterator of transformed value
 */
export async function* asyncMap<T, O>(it: AsyncIterable<T>, callback: (data: T, index?: number) => Promise<O>) {
    if (!it) return;

    let index = 0;

    for await (const v of it) {
        yield await callback(v, index++);
    }
}

/**
 * Get the first element out of an async iterator
 * @param it The iterator to take element from
 * @returns The first element
 */
export async function asyncFirst<T>(it: AsyncIterable<T>): Promise<T> {
    for await (const v of it) {
        return v;
    }
}

/**
 * Drain an Async iterator into an Array
 * @param it An iterator to complete
 * @returns An array of all the value
 */
export async function asyncToArray<T>(it: AsyncIterable<T>): Promise<T[]> {
    const out = [];

    for await (const v of it) {
        out.push(v);
    }

    return out;
}

/**
 * Flatmap over an Iterators values, return a new iterator based on value, that iterator is then exhausted inline to 
 * "flatten" then stream 
 * @param it Iterator
 * @param callback Function to transform the value in the iterator
 */
export function* flatMap<T, O>(it: Iterable<T>, callback: (data: T, index?: number) => Iterable<O>) {
    if (!it) return;

    let index = 0;

    for (const v of it) {
        const it2 = callback(v, index++);
        for (const v2 of it2) {
            yield v2;
        }
    }
}

/**
 * Take only n amount of values from the iterator, it read the iterator to N or end depending which comes first
 * @param it Iterator
 * @param n The number of values to read
 */
export function* take<T>(it: Iterator<T>, n: number) {
    for (let i = 0; i < n; i++) {
        const r = it.next();

        if (r.done === true) {
            return;
        }

        yield r.value;
    }
}

/**
 * Pop the next value in the iterator, to go from iterator world to reified values
 * A convenience function
 * @param it Iterator
 * @returns The value or undefined if the Iterator was empty
 */
export function pop<T>(it: Iterator<T>): T | undefined {
    if (!it) return undefined;

    const n = it.next();

    if (n.done === true) {
        return undefined;
    }
    
    return n.value;
}   

/**
 * Repeat a value n number of times before the Generator is exhausted
 * @param value The value to repeat
 * @param n The number of times to repeat
 */
export function* repeat<T>(value: T, n: number) {
    for(let i = 0; i < n; i++) {
        yield value;
    }
}

/**
 * Chain multiple Iterators togeather forming one big Iterator that exhausts the sub-iterators one by one
 * @param args The iterators to add together
 */
export function* chain<T>(...args: Iterable<T>[]){
    for(const it of args) {
        for(const v of it) {
            yield v;
        }
    }
}

/**
 * Filter an iterator with the callback predicate, calls returning true are kept
 * @param it A base iterator
 * @param callback A predicate that decides if the value is worth it
 */
export function* filter<T>(it: Iterable<T>, callback: (val: T) => boolean) {
    for (const v of it) {
        if (callback(v)) {
            yield v;
        }
    }
}

/**
 * Iterate of an iterator using a sliding window aproach.
 * Values a going to be yielded in groups of N until the iterator is exhausted
 * @param it Iterator
 * @param size The size of the window
 */
export function* window<T>(it: IterableIterator<T>, size: number) {
    let stack = Array.from(take(it, size));

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

    const first = pop(it);

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
