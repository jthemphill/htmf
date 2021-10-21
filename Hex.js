import * as React from 'react';
import Penguin from './Penguin';
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
export default React.memo(function (props) {
    const transform = `translate(${props.cx},${props.cy})`;
    const player = props.player;
    if (player === undefined && props.claimed) {
        return (React.createElement("g", { transform: transform },
            React.createElement("polygon", { points: points(props).join(' '), className: "cell claimed" })));
    }
    let penguin = undefined;
    let circles = undefined;
    if (player !== undefined) {
        penguin = (React.createElement(Penguin, { player: player, size: props.sideLength }));
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