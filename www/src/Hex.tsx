import * as React from "react";

import Penguin from "./Penguin";

import "./index.css";

interface Props {
  _key: number;
  onClick: (key: number) => void;

  claimed: boolean;
  fish: number;
  cx: number;
  cy: number;
  sideLength: number;
  highlighted: boolean;
  possible: boolean;
  player?: number;
  isTopMoveSrc: boolean;
  isTopMoveDst: boolean;
}

function points(sideLength: number): number[][] {
  const points = [];
  for (let i = 0; i < 6; ++i) {
    const angleRadians = (i * Math.PI) / 3;
    points.push([
      sideLength * Math.sin(angleRadians),
      sideLength * Math.cos(angleRadians),
    ]);
  }
  return points;
}

function Circles({
  sideLength,
  fish,
}: {
  sideLength: number;
  fish: number;
}): React.JSX.Element[] {
  const r = sideLength / 10;
  if (fish === 1) {
    return [<circle key={0} className="fish" r={r} />];
  } else if (fish === 2) {
    return [
      <circle key={0} className="fish" cx={-r * 2} r={r} />,
      <circle key={1} className="fish" cx={r * 2} r={r} />,
    ];
  } else if (fish === 3) {
    const yOffset = r * 2 * Math.sin(Math.PI / 3);
    return [
      <circle key={0} className="fish" cx={-r * 2} cy={yOffset} r={r} />,
      <circle key={1} className="fish" cx={r * 2} cy={yOffset} r={r} />,
      <circle key={2} className="fish" cx={0} cy={-yOffset} r={r} />,
    ];
  } else {
    throw new Error(`${fish.toString()} is not a valid fish amount`);
  }
}

export default function Hex({
  _key,
  sideLength,
  highlighted,
  possible,
  player,
  fish,
  claimed,
  cx,
  cy,
  onClick,
  isTopMoveSrc,
  isTopMoveDst,
}: Props): React.JSX.Element {
  function handleClick() {
    onClick(_key);
  }

  const transform = `translate(${cx.toString()},${cy.toString()})`;

  if (player === undefined && claimed) {
    return (
      <g transform={transform}>
        <polygon
          points={points(sideLength).join(" ")}
          className="cell claimed"
        />
      </g>
    );
  }

  let penguin;
  let circles;
  if (player !== undefined) {
    penguin = <Penguin player={player} size={sideLength} />;
  } else {
    circles = <Circles sideLength={sideLength} fish={fish} />;
  }
  const cellClasses = ["cell"];
  if (highlighted) {
    cellClasses.push("highlighted");
  } else if (possible) {
    cellClasses.push("possible");
  }
  if (isTopMoveSrc) {
    cellClasses.push("top-move-src");
  } else if (isTopMoveDst) {
    cellClasses.push("top-move-dst");
  }
  return (
    <g
      data-testid={`Hex::${_key.toString()}`}
      className={cellClasses.join(" ")}
      role="button"
      aria-disabled={!possible}
      transform={transform}
      onClick={handleClick}
    >
      <polygon points={points(sideLength).join(" ")} />
      {circles}
      {penguin}
    </g>
  );
}
