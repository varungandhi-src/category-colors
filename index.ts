import chroma from "chroma-js";

const modes = ["light", "dark"];

const backgroundColors = {
	"dark": "#303346",
	"light": "#ffffff"
};

// From
// https://handbook.sourcegraph.com/departments/engineering/product/design/brand_guidelines/color/#secondary-colors
//
// These also include the primary colors.
const brandColors = {
    "mist": [
        "#fff2cf",
        "#ffc9c9",
        "#ffd1f2",
        "#e8d1ff",
        "#bfbfff",
        "#c7ffff"
    ],
    "light": [
        "#ffdb45",
        "#ff5543",
        "#d62687",
        "#a112ff",
        "#6b59ed",
        "#00cbec",
        "#8fedcf"
    ],
    "medium": [
        "#ffc247",
        "#ed2e20",
        "#c4147d",
        "#820dde",
        "#5033E1",
        "#00a1c7",
        "#17ab52"
    ],
    "dark": [
        "#ff9933",
        "#c22626",
        "#9e1769",
        "#6112a3",
        "#3826cc",
        "#005482",
        "#1f7d45"
    ]
};

const alertColors = [
    "#82a460",
    "#c3c865",
    "#bb3926"
];

const targetColors = [
    "#9966FF",
    "#0055BC",
    "#00A1C2",
    "#ED6804",
    "#B3063D"
];

// random from array
const randomFromArray = (array: number[]) => {
    return array[Math.floor(Math.random() * array.length)];
};

// generate a random color
const randomColor = (): chroma.Color => {
    const color = chroma.random();
    return color;
};

type Color = string | chroma.Color

// measures the distance between two colors
const distance = (color1: Color, color2: Color): number => chroma.deltaE(color1, color2);

const getClosestColor = (color: Color, colorArray: Color[]): Color => {
    const distances = colorArray.map((c) => distance(color, c));
    const minIndex = distances.indexOf(Math.min(...distances));
    return colorArray[minIndex];
};

const sgDistances = (backgroundColor: chroma.Color, foregroundColors: chroma.Color[], visionSpace = "Normal"): {fromBg: number[], mutual: number[]} => {
    let bgDistances: number[] = [];
    let distances: number[] = [];
    const convertedBgColor: Color = brettelFunctions[visionSpace](backgroundColor.rgb());
    const convertedColors: Color[] = foregroundColors.map((c) =>
        brettelFunctions[visionSpace](c.rgb())
    );
    for (let i = 0; i < foregroundColors.length; i++) {
        bgDistances.push(distance(convertedBgColor, convertedColors[i]));
        for (let j = i + 1; j < foregroundColors.length; j++) {
            distances.push(distance(convertedColors[i], convertedColors[j]));
        }
    }
    return {fromBg: bgDistances, mutual: distances};
}

// get average of interger array
const average = (array: number[]) => array.reduce((a, b) => a + b) / array.length;


// get the distance between the highest and lowest values in an array
const range = (array: number[]) => {
    const sorted = array.sort((a, b) => a - b);
    return sorted[sorted.length - 1] - sorted[0];
};

// produces a color a small random distance away from the given color
const randomNearbyColor = (color: chroma.Color): chroma.Color => {
    const channelToChange = randomFromArray([0, 1, 2]);
    const oldVal = color.gl()[channelToChange];
    let newVal = oldVal + Math.random() * 0.1 - 0.05;
    if (newVal > 1) {
        newVal = 1;
    } else if (newVal < 0) {
        newVal = 0;
    }
    return color.set(`rgb.${"rgb"[channelToChange]}`, newVal * 255);
};

// average of distances between array of colors and target colors
const averageDistanceFromTargetColors = (colors: chroma.Color[], targetColors: chroma.Color[]): number => {
    const distances = colors.map((c) =>
        distance(c, getClosestColor(c, targetColors))
    );
    return average(distances);
};

// Bretel et al method for simulating color vision deficiency
// Adapted from https://github.com/MaPePeR/jsColorblindSimulator
// In turn adapted from libDaltonLens https://daltonlens.org (public domain) 

// convert a linear rgb value to sRGB
const linearRGB_from_sRGB = (v: number): number => {
    var fv = v / 255.0;
    if (fv < 0.04045) return fv / 12.92;
    return Math.pow((fv + 0.055) / 1.055, 2.4);
}

const sRGB_from_linearRGB = (v: number): number => {
    if (v <= 0) return 0;
    if (v >= 1) return 255;
    if (v < 0.0031308) return 0.5 + v * 12.92 * 255;
    return 0 + 255 * (Math.pow(v, 1.0 / 2.4) * 1.055 - 0.055);
}

const brettelFunctions = {
    Normal: function (v) {
        return v;
    },
    Protanopia: function (v) {
        return brettel(v, "protan", 1.0);
    },
    Protanomaly: function (v) {
        return brettel(v, "protan", 0.6);
    },
    Deuteranopia: function (v) {
        return brettel(v, "deutan", 1.0);
    },
    Deuteranomaly: function (v) {
        return brettel(v, "deutan", 0.6);
    },
    Tritanopia: function (v) {
        return brettel(v, "tritan", 1.0);
    },
    Tritanomaly: function (v) {
        return brettel(v, "tritan", 0.6);
    },
    Achromatopsia: function (v) {
        return monochrome_with_severity(v, 1.0);
    },
    Achromatomaly: function (v) {
        return monochrome_with_severity(v, 0.6);
    },
};

var sRGB_to_linearRGB_Lookup: number[] = Array(256);
(function () {
    for (let i = 0; i < 256; i++) {
        sRGB_to_linearRGB_Lookup[i] = linearRGB_from_sRGB(i);
    }
})();

const brettel_params = {
    protan: {
        rgbCvdFromRgb_1: [
            0.1451, 1.20165, -0.34675, 0.10447, 0.85316, 0.04237, 0.00429,
            -0.00603, 1.00174,
        ],
        rgbCvdFromRgb_2: [
            0.14115, 1.16782, -0.30897, 0.10495, 0.8573, 0.03776, 0.00431,
            -0.00586, 1.00155,
        ],
        separationPlaneNormal: [0.00048, 0.00416, -0.00464],
    },

    deutan: {
        rgbCvdFromRgb_1: [
            0.36198, 0.86755, -0.22953, 0.26099, 0.64512, 0.09389, -0.01975,
            0.02686, 0.99289,
        ],
        rgbCvdFromRgb_2: [
            0.37009, 0.8854, -0.25549, 0.25767, 0.63782, 0.10451, -0.0195,
            0.02741, 0.99209,
        ],
        separationPlaneNormal: [-0.00293, -0.00645, 0.00938],
    },

    tritan: {
        rgbCvdFromRgb_1: [
            1.01354, 0.14268, -0.15622, -0.01181, 0.87561, 0.13619, 0.07707,
            0.81208, 0.11085,
        ],
        rgbCvdFromRgb_2: [
            0.93337, 0.19999, -0.13336, 0.05809, 0.82565, 0.11626, -0.37923,
            1.13825, 0.24098,
        ],
        separationPlaneNormal: [0.0396, -0.02831, -0.01129],
    },
};

function brettel(srgb, t, severity): any {
    // Go from sRGB to linearRGB
    var rgb = Array(3);
    rgb[0] = sRGB_to_linearRGB_Lookup[srgb[0]];
    rgb[1] = sRGB_to_linearRGB_Lookup[srgb[1]];
    rgb[2] = sRGB_to_linearRGB_Lookup[srgb[2]];

    var params = brettel_params[t];
    var separationPlaneNormal = params["separationPlaneNormal"];
    var rgbCvdFromRgb_1 = params["rgbCvdFromRgb_1"];
    var rgbCvdFromRgb_2 = params["rgbCvdFromRgb_2"];

    // Check on which plane we should project by comparing wih the separation plane normal.
    var dotWithSepPlane =
        rgb[0] * separationPlaneNormal[0] +
        rgb[1] * separationPlaneNormal[1] +
        rgb[2] * separationPlaneNormal[2];
    var rgbCvdFromRgb =
        dotWithSepPlane >= 0 ? rgbCvdFromRgb_1 : rgbCvdFromRgb_2;

    // Transform to the full dichromat projection plane.
    var rgb_cvd = Array(3);
    rgb_cvd[0] =
        rgbCvdFromRgb[0] * rgb[0] +
        rgbCvdFromRgb[1] * rgb[1] +
        rgbCvdFromRgb[2] * rgb[2];
    rgb_cvd[1] =
        rgbCvdFromRgb[3] * rgb[0] +
        rgbCvdFromRgb[4] * rgb[1] +
        rgbCvdFromRgb[5] * rgb[2];
    rgb_cvd[2] =
        rgbCvdFromRgb[6] * rgb[0] +
        rgbCvdFromRgb[7] * rgb[1] +
        rgbCvdFromRgb[8] * rgb[2];

    // Apply the severity factor as a linear interpolation.
    // It's the same to do it in the RGB space or in the LMS
    // space since it's a linear transform.
    rgb_cvd[0] = rgb_cvd[0] * severity + rgb[0] * (1.0 - severity);
    rgb_cvd[1] = rgb_cvd[1] * severity + rgb[1] * (1.0 - severity);
    rgb_cvd[2] = rgb_cvd[2] * severity + rgb[2] * (1.0 - severity);

    // Go back to sRGB
    return [
        sRGB_from_linearRGB(rgb_cvd[0]),
        sRGB_from_linearRGB(rgb_cvd[1]),
        sRGB_from_linearRGB(rgb_cvd[2]),
    ];
}

// Adjusted from the hcirn code
function monochrome_with_severity(srgb: number[], severity: number): number[] {
    var z = Math.round(srgb[0] * 0.299 + srgb[1] * 0.587 + srgb[2] * 0.114);
    var r = z * severity + (1.0 - severity) * srgb[0];
    var g = z * severity + (1.0 - severity) * srgb[1];
    var b = z * severity + (1.0 - severity) * srgb[2];
    return [r, g, b];
}

function rms(xs: number[]): number {
    return Math.sqrt(average(xs.map((x) => x * x)))
}

function rms_distance(baseline: number, xs: number[]): number {
    return rms(xs.map((x) => (baseline - x)))
}

// Cost function including weights
const cost = (state: chroma.Color[], bgColor: chroma.Color, targetColors: chroma.Color[]): number => {
    const energyWeight = 1;
    const rangeWeight = 0.5;
    const targetWeight = 1;
    const protanopiaWeight = 0.33;
    const deuteranopiaWeight = 0.33;
    const tritanopiaWeight = 0.33;

    const normalDistances = sgDistances(bgColor, state);
    const protanopiaDistances = sgDistances(bgColor, state, "Protanopia");
    const deuteranopiaDistances = sgDistances(bgColor, state, "Deuteranopia");
    const tritanopiaDistances = sgDistances(bgColor, state, "Tritanopia");

    const costFrom = (distances: {fromBg: number[], mutual: number[]}): number => {
        const bgWeight = 0.2;
        const bgScore = rms_distance(100, distances.fromBg);
        const mutualScore = rms_distance(100, distances.mutual);
        return bgScore * bgWeight + (1 - bgWeight) * mutualScore
    }

    const energyCost =  costFrom(normalDistances); 
    const protanopiaCost = costFrom(protanopiaDistances);
    const deuteranopiaCost = costFrom(deuteranopiaDistances);
    const tritanopiaCost = costFrom(tritanopiaDistances);
    // This should really be variance
    const rangeCost = range(normalDistances.mutual);
    const targetCost = averageDistanceFromTargetColors(state, targetColors);

    return (
        energyWeight * energyCost +
        targetWeight * targetCost +
        rangeWeight * rangeCost +
        protanopiaWeight * protanopiaCost +
        deuteranopiaWeight * deuteranopiaCost +
        tritanopiaWeight * tritanopiaCost
    );
};

// the simulated annealing algorithm
const optimize = (n = 5, bgColor: chroma.Color, targetColors: chroma.Color[]) => {
    let colors: chroma.Color[] = [];
    for (let i = 0; i < n; i++) {
        colors.push(randomColor());
    }

    const calculateCost = (current: chroma.Color[]): number => { return cost(current, bgColor, targetColors) }

    const startColors = Array.from(targetColors); // is this a good starting point?
    const startCost = calculateCost(startColors);
    let oldCost = startCost;

    // intialize hyperparameters
    let temperature = 1000;
    const coolingRate = 0.99;
    const cutoff = 0.0001;

    // iteration loop
    while (temperature > cutoff) {
        // for each color
        for (let i = 0; i < colors.length; i++) {
            // copy old colors
            const newColors = colors.map((color) => color);
            // move the current color randomly
            newColors[i] = randomNearbyColor(newColors[i]);
            // choose between the current state and the new state
            // based on the difference between the two, the temperature
            // of the algorithm, and some random chance
            const newCost = calculateCost(newColors);
            const delta = newCost - oldCost;
            const probability = Math.exp(-delta / temperature);
            if (Math.random() < probability) {
                colors[i] = newColors[i];
                oldCost = newCost;
            }
        }

        // decrease temperature
        temperature *= coolingRate;
    }

    console.log(`
Start colors: ${startColors.map((color) => color.hex())}
Start cost: ${startCost}
Final colors: ${colors.map((color) => color.hex())}
Final cost: ${calculateCost(colors)}
Cost difference: ${calculateCost(colors) - startCost}`);
    return colors;
};


const sgMain = () => {
    for (const mode of modes) {
        // Use fewer secondaries for now, optimization with lots of colors takes long.
        const secondaries = (mode === "dark") ? ["mist", "light"] : ["light", "medium"];

        // const secondaries = (mode == "dark") ?
        //    ["mist", "light", "medium"] : ["light", "medium", "dark"];
        let targets: string[] = [];
        for (const key of secondaries) {
            targets.push(...brandColors[key]);
        }
        targets.push(...alertColors);

        const targetColors = targets.map((cstr) => { return chroma(cstr) })
        const str = `Optimized for ${mode} mode with background color ${backgroundColors[mode]}`
        console.time(str);
        optimize(targetColors.length, chroma(backgroundColors[mode]), targetColors);
        console.timeEnd(str);
    }
}

sgMain();