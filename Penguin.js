"use strict";
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
Object.defineProperty(exports, "__esModule", { value: true });
const React = __importStar(require("react"));
const constants_1 = require("./constants");
exports.default = React.memo(function Penguin(props) {
    let playerClass = undefined;
    if (props.player == constants_1.HUMAN_PLAYER) {
        playerClass = "human";
    }
    else if (props.player == constants_1.BOT_PLAYER) {
        playerClass = "bot";
    }
    const solidPartClass = `penguin ${playerClass}`;
    const scale = props.size / 500;
    const translateX = -307.62496;
    const translateY = -350;
    return (React.createElement("g", { transform: `scale(${scale}),translate(${translateX},${translateY})` },
        React.createElement("g", { transform: "matrix(2.0165499,0,0,2.0165499,-378.20444,-374.28247)" },
            React.createElement("g", { transform: "translate(0,-14)", className: "penguin leftFoot" },
                React.createElement("rect", { className: "penguin foot", width: "18.182745", height: "72.73098", x: "340.37064", y: "378.86526" }),
                React.createElement("path", { className: "penguin foot", d: "m 347.12481,446.95339 -60.42857,21.85714 45.71428,18.57143 26.14286,-35.78572 z" })),
            React.createElement("g", { transform: "matrix(-1,0,0,1,745.35258,-14)", className: "penguin rightFoot" },
                React.createElement("rect", { y: "378.86526", x: "340.37064", height: "72.73098", width: "18.182745", className: "penguin foot" }),
                React.createElement("path", { d: "m 347.12481,446.95339 -60.42857,21.85714 45.71428,18.57143 26.14286,-35.78572 z", className: "penguin foot" })),
            React.createElement("path", { transform: "matrix(0.59003831,0,0,0.59003831,186.08628,108.73722)", d: "m 446.48743,416.47116 c 0,72.80489 -58.79389,131.82491 -131.31983,131.82491 -72.52593,0 -131.31982,-59.02002 -131.31982,-131.82491 0,-72.80488 58.79389,-131.8249 131.31982,-131.8249 72.52594,0 131.31983,59.02002 131.31983,131.8249 z", className: solidPartClass }),
            React.createElement("path", { transform: "translate(59.910095,0)", d: "m 361.63462,289.69702 c 0,27.3367 -22.16077,49.49748 -49.49747,49.49748 -27.3367,0 -49.49748,-22.16078 -49.49748,-49.49748 0,-27.3367 22.16078,-49.49747 49.49748,-49.49747 27.3367,0 49.49747,22.16077 49.49747,49.49747 z", className: solidPartClass }),
            React.createElement("path", { d: "m 345.78328,275.11088 c 0,0 6.9297,-6 26.26396,-6 19.33426,0 26.26397,6 26.26397,6 l -26.26397,52.52793 z", className: "penguin beak" }),
            React.createElement("path", { transform: "translate(0,-8)", className: "penguin eyes", d: "m 369.46331,266.71603 c 0,5.43945 -4.40955,9.84899 -9.84899,9.84899 -5.43945,0 -9.84899,-4.40954 -9.84899,-9.84899 0,-5.43944 4.40954,-9.84898 9.84899,-9.84898 5.43944,0 9.84899,4.40954 9.84899,9.84898 z" }),
            React.createElement("path", { transform: "translate(0,-8)", d: "m 391.93917,269.36768 c 0,3.55656 -2.88316,6.43972 -6.43972,6.43972 -3.55656,0 -6.43972,-2.88316 -6.43972,-6.43972 0,-3.55656 2.88316,-6.43973 6.43972,-6.43973 3.55656,0 6.43972,2.88317 6.43972,6.43973 z", className: "penguin eyes" }),
            React.createElement("path", { d: "M 332.14074,297.03867 247.29889,260.5461 c 10.94712,32.37883 32.49618,64.01962 75.63693,75.93783 z", className: solidPartClass }),
            React.createElement("path", { className: solidPartClass, d: "M 411.95375,297.03867 496.7956,260.5461 c -10.94712,32.37883 -32.49618,64.01962 -75.63693,75.93783 z" }))));
});
//# sourceMappingURL=Penguin.js.map