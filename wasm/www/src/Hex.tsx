import * as React from 'react';

import Penguin from './Penguin';

type Props = {
    _key: number,
    onClick: ((key: number) => void),

    claimed: boolean,
    fish: number,
    cx: number,
    cy: number,
    sideLength: number,
    highlighted: boolean,
    possible: boolean,
    player?: number,
};

function points(props: Props): Array<Array<number>> {
    const points = [];
    for (let i = 0; i < 6; ++i) {
        points.push(corner(props, i));
    }
    return points;
}

function corner(props: Props, i: number): Array<number> {
    const side_length = props.sideLength;
    const angle_rad = i * Math.PI / 3;
    return [
        side_length * Math.sin(angle_rad),
        side_length * Math.cos(angle_rad),
    ];
}

function draw_circles(props: Props) {
    const r = props.sideLength / 10;
    if (props.fish === 1) {
        return [
            <circle
                key={0}
                className="fish"
                r={r}
            />,
        ];
    } else if (props.fish === 2) {
        return [
            <circle
                key={0}
                className="fish"
                cx={-r * 2}
                r={r}
            />,
            <circle
                key={1}
                className="fish"
                cx={r * 2}
                r={r}
            />,
        ];
    } else if (props.fish === 3) {
        const y_offset = r * 2 * Math.sin(Math.PI / 3)
        return [
            <circle
                key={0}
                className="fish"
                cx={-r * 2}
                cy={y_offset}
                r={r}
            />,
            <circle
                key={1}
                className="fish"
                cx={r * 2}
                cy={y_offset}
                r={r}
            />,
            <circle
                key={2}
                className="fish"
                cx={0}
                cy={-y_offset}
                r={r}
            />,
        ];
    } else {
        throw new Error(`${props.fish} is not a valid fish amount`);
    }
}

const Hex = React.memo(function (props: Props) {
    const transform = `translate(${props.cx},${props.cy})`;

    const player = props.player;
    if (player === undefined && props.claimed) {
        return (
            <g transform={transform}>
                <polygon
                    points={points(props).join(' ')}
                    className="cell claimed"
                />
            </g>
        );
    }

    let penguin = undefined;
    let circles = undefined;
    if (player !== undefined) {
        penguin = (<Penguin player={player} size={props.sideLength} />);
    } else {
        circles = draw_circles(props);
    }
    const cellClasses = ["cell"];
    if (props.highlighted) {
        cellClasses.push("highlighted");
    } else if (props.possible) {
        cellClasses.push("possible");
    }
    return (
        <g transform={transform} onClick={() => props.onClick(props._key)}>
            <polygon
                className={cellClasses.join(" ")}
                points={points(props).join(" ")}
            />
            {circles}
            {penguin}
        </g>
    );
});

export default Hex;