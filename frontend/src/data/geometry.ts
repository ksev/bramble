/**
 * A point in two dimentional space
 */
export class Point {
    static readonly ZERO = new Point(0, 0);

    constructor(public readonly x: number, public readonly y: number) {}

    equals(other: Point): boolean {
        return this.x === other.x && this.y === other.y;
    }
}

/**
 * A size of something with two dimentional axies
 */
export class Extent {
    constructor(public readonly width: number, public readonly height: number) {}
}

/**
 * A rectangle
 */
export class Rect {
    constructor(public readonly origin: Point, public readonly size: Extent) {}
    
    /**
     * Create a new rect with a different origin but the same size
     * @param origin The new origin
     * @returns A new rect
     */
    moveTo(origin: Point) {
        return new Rect(origin, this.size);
    }

    /**
     * Resize the rect
     * @param extent The new size the rect should have
     * @returns A new square with the same origin but a diffenent size
     */
    resize(extent: Extent) {
        return new Rect(this.origin, extent);
    }
}

