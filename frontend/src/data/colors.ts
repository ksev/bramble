import Color from 'color';

export default {
    background: Color("#1f1f33"),
    strong: Color("#fafafa"),
    container: Color("rgb(56 56 76)"),
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

export { Color };
