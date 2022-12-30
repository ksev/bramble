/**
 * A simple class that implements Promise but can be resolved/rejected by calling instance members instead of callbacks in the 
 * constructor, sometimes useful when the dataflow can't easily be expressed with the default and recommended way of making Promises
 */
export class MutablePromise<T, E = any> implements Promise<T> {
    private promise: Promise<T>;

    resolve: (data: T) => void;
    reject: (reason: E) => void;

    constructor() {
        this.promise = new Promise((resolve, reject) => {
            this.resolve = resolve;
            this.reject = reject;
        });
    }   

    then<TResult1 = T, TResult2 = never>(onfulfilled?: (value: T) => TResult1 | PromiseLike<TResult1>, onrejected?: (reason: any) => TResult2 | PromiseLike<TResult2>): Promise<TResult1> {
        return this.promise.then(onfulfilled, onrejected) as Promise<TResult1>;
    }

    catch<TResult>(onrejected?: (reason: any) => TResult | PromiseLike<TResult>): Promise<TResult> {
        return this.promise.catch(onrejected) as Promise<TResult>;
    }

    finally(onfinally: () => void): Promise<T> {
        return this.promise.finally(onfinally);
    }

    [Symbol.toStringTag]: "[object MutablePromise]"
}

/**
 * Generate a 128-bit random string id, NOT uuid more compact
 * @returns 
 */
export function randomId(): string {
    const buffer = new Uint32Array(4);
    crypto.getRandomValues(buffer);
    return base64CompactArrayBuffer(buffer.buffer);
}

/**
 * Convert an arrayBuffer into base64 without the end padding
 * @param arrayBuffer Array buffer to encode
 * @returns A base64 encoded string
 */
export function base64CompactArrayBuffer(arrayBuffer: ArrayBuffer) {
    var base64 = ''
    var encodings = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

    var bytes = new Uint8Array(arrayBuffer)
    var byteLength = bytes.byteLength
    var byteRemainder = byteLength % 3
    var mainLength = byteLength - byteRemainder

    var a: number, b: number, c: number, d: number
    var chunk: number;

    // Main loop deals with bytes in chunks of 3
    for (var i = 0; i < mainLength; i = i + 3) {
        // Combine the three bytes into a single integer
        chunk = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2]

        // Use bitmasks to extract 6-bit segments from the triplet
        a = (chunk & 16515072) >> 18 // 16515072 = (2^6 - 1) << 18
        b = (chunk & 258048) >> 12 // 258048   = (2^6 - 1) << 12
        c = (chunk & 4032) >> 6 // 4032     = (2^6 - 1) << 6
        d = chunk & 63               // 63       = 2^6 - 1

        // Convert the raw binary segments to the appropriate ASCII encoding
        base64 += encodings[a] + encodings[b] + encodings[c] + encodings[d]
    }

    // Deal with the remaining bytes and padding
    if (byteRemainder == 1) {
        chunk = bytes[mainLength]

        a = (chunk & 252) >> 2 // 252 = (2^6 - 1) << 2

        // Set the 4 least significant bits to zero
        b = (chunk & 3) << 4 // 3   = 2^2 - 1

        base64 += encodings[a] + encodings[b]
    } else if (byteRemainder == 2) {
        chunk = (bytes[mainLength] << 8) | bytes[mainLength + 1]

        a = (chunk & 64512) >> 10 // 64512 = (2^6 - 1) << 10
        b = (chunk & 1008) >> 4 // 1008  = (2^6 - 1) << 4

        // Set the 2 least significant bits to zero
        c = (chunk & 15) << 2 // 15    = 2^4 - 1

        base64 += encodings[a] + encodings[b] + encodings[c]
    }

    return base64;
}