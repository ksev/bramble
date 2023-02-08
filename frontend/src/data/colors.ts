import Color from 'color';
import { ValueKind } from './api-gen/api_types';

const colors = {
    success: Color("rgb(112, 170, 100)"),
    background: Color("#1f1f33"),
    strong: Color("#fafafa"),
    container: Color("rgb(56,56,76)"),
    containerHigh: Color("rgb(72, 72, 96)"),    
    fadedtext: Color("rgb(159, 161, 172)"),
    icon: Color("#b7bac8"),
    device: Color("#6cb6ff"),
    feature: Color("#6c6dff"),
    automation: Color("rgb(16, 148, 213)"),
    source: Color("#6cb6ff"),
    sink: Color("#ff6c6d"),
    sourceSink: Color("#b46cff"),
    transparent: Color('rgba(0,0,0,0)'),
    error: Color("rgb(244, 98, 98)"),

    // Colors for the different value types
    bool: Color("rgb(140, 108, 255)"),
    number: Color("#46a2ce"),
    state: Color("rgb(255, 156, 108)"),
    string: Color("#ff6c6d"),
    any: Color("rgb(140, 108, 255)").mix(Color("#46a2ce"), 0.5).mix(Color("rgb(255, 156, 108)"), 0.5).mix(Color("#ff6c6d"), 0.5),
}

export function kindColor(kind: ValueKind | "ANY"): Color {
    switch(kind) {
        case "BOOL": return colors.bool;
        case "NUMBER": return colors.number;
        case "STATE": return colors.state;
        case "STRING": return colors.string;
        case "ANY": return colors.any;
    }
}

export function directionColor(direction: "SOURCE" | "SINK" | "SOURCE_SINK"): Color {
    switch (direction) {
        case "SOURCE": return colors.source;
        case "SINK": return colors.sink;
        case "SOURCE_SINK": return colors.sourceSink;
    }
}

export { Color };

export default colors;
