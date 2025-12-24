import * as React from "react";
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
export default function Hex({ _key, sideLength, highlighted, possible, player, fish, claimed, cx, cy, onClick, isTopMoveSrc, isTopMoveDst, }: Props): React.JSX.Element;
export {};
