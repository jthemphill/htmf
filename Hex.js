"use strict";
// @flow
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const React = __importStar(require("react"));
const Penguin_1 = __importDefault(require("./Penguin"));
function points(props) {
    const points = [];
    for (let i = 0; i < 6; ++i) {
        points.push(corner(props, i));
    }
    return points;
}
function corner(props, i) {
    const side_length = props.sideLength;
    const angle_rad = i * Math.PI / 3;
    return [
        side_length * Math.sin(angle_rad),
        side_length * Math.cos(angle_rad),
    ];
}
function draw_circles(props) {
    const r = props.sideLength / 10;
    if (props.fish === 1) {
        return [
            React.createElement("circle", { key: 0, className: "fish", r: r }),
        ];
    }
    else if (props.fish === 2) {
        return [
            React.createElement("circle", { key: 0, className: "fish", cx: -r * 2, r: r }),
            React.createElement("circle", { key: 1, className: "fish", cx: r * 2, r: r }),
        ];
    }
    else if (props.fish === 3) {
        const y_offset = r * 2 * Math.sin(Math.PI / 3);
        return [
            React.createElement("circle", { key: 0, className: "fish", cx: -r * 2, cy: y_offset, r: r }),
            React.createElement("circle", { key: 1, className: "fish", cx: r * 2, cy: y_offset, r: r }),
            React.createElement("circle", { key: 2, className: "fish", cx: 0, cy: -y_offset, r: r }),
        ];
    }
    else {
        throw new Error(`${props.fish} is not a valid fish amount`);
    }
}
exports.default = React.memo(function (props) {
    const transform = `translate(${props.cx},${props.cy})`;
    const color = props.color;
    if (!color && props.claimed) {
        return (React.createElement("g", { transform: transform },
            React.createElement("polygon", { points: points(props).join(' '), className: "cell claimed" })));
    }
    let penguin = null;
    let circles = null;
    if (color) {
        penguin = (React.createElement(Penguin_1.default, { color: color, size: props.sideLength }));
    }
    else {
        circles = draw_circles(props);
    }
    const cellClasses = ["cell"];
    if (props.highlighted) {
        cellClasses.push("highlighted");
    }
    else if (props.possible) {
        cellClasses.push("possible");
    }
    return (React.createElement("g", { transform: transform, onClick: () => props.onClick(props._key) },
        React.createElement("polygon", { className: cellClasses.join(" "), points: points(props).join(" ") }),
        circles,
        penguin));
});
//# sourceMappingURL=Hex.js.map