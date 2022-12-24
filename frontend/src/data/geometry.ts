/**
 * A point in two dimentional space
 */
export class Point {
    static readonly ZERO = new Point(0, 0);

    constructor(public readonly x: number, public readonly y: number) {}

    /**
     * Get the squared distance between two points
     * @param other Some toher
     * @returns Squared distance between points
     */
    distanceSquared(other: Point): number {
        let a = Math.pow(this.x - other.x, 2);
        let b = Math.pow(this.y - other.y, 2);
        return a + b;
    }

    /**
     * Check if this Point and another Point are value equal
     * @param other Some other point
     * @returns True if the are equal by value
     */
    equals(other: Point): boolean {
        return this.x === other.x && this.y === other.y;
    }
}

/**
 * A size of something with two dimentional axies
 */
export class Extent {
    static readonly ZERO = new Extent(0, 0);

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

    /**
     * The union between two rects, ie a new rect that fits both
     * @param other Other rect
     * @returns A new rect of the same size or larger
     */
     union(other: Rect): Rect {
        if (!other) return this;

        const origin = new Point(
            Math.min(this.origin.x, other.origin.x),
            Math.min(this.origin.y, other.origin.y)
        );

        const extent = new Extent(
            Math.max(this.origin.x+this.size.width, other.origin.x+other.size.width),
            Math.max(this.origin.y+this.size.height, other.origin.y+other.size.height)
        );

        return new Rect(origin, extent);
    }

    /**
     * Return a new rect where the two rects overlap
     * @param other Another rect
     */
    intersect(other: Rect): Rect | undefined {
        if (!other) return undefined;

        let x1 = Math.max(this.origin.x, other.origin.x);
        let x2 = Math.min(this.origin.x + this.size.width, other.origin.x + other.size.width);

        if (x2 <= x1) { return undefined; }
        
        let y1 = Math.max(this.origin.y, other.origin.y);
        let y2 = Math.min(this.origin.y + this.size.height, other.origin.y + other.size.height);

        if (y2 <= y1) { return undefined; }

        return new Rect(
            new Point(x1, y1), 
            new Extent(
                x2 - x1,
                y2 - y1
            )
        );
    }

    /**
     * Create a rect from just the numbers
     * @param x X position
     * @param y Y position
     * @param width The width
     * @param height The height
     * @returns A rect
     */
    static numbers(x: number, y: number, width: number, height: number) {
        return new Rect(
            new Point(x, y),
            new Extent(width, height)
        )
    }

    /**
     * Create a rect from two corner points
     * @param c1 First corner
     * @param c2 Second corner
     */
    static corners(c1: Point, c2: Point): Rect {
        let minX = Math.min(c1.x, c2.x);
        let minY = Math.min(c1.y, c2.y);

        let maxX = Math.max(c1.x, c2.x);
        let maxY = Math.max(c1.y, c2.y);

        return new Rect(
            new Point(minX, minY),
            new Extent(
                maxX - minX,
                maxY - minY
            )
        );
    }
}

