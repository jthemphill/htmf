// @flow

import * as React from 'react';

import Penguin from './Penguin.react';

type Props = {
    _key: number,
    onClick: ((key: number) => void),

    claimed: bool,
    fish: number,
    cx: number,
    cy: number,
    sideLength: number,
    highlighted: bool,
    possible: bool,
    color: ?string,
};

class Hex extends React.PureComponent<Props> {

    render() {
        const transform = 'translate(' + this.props.cx + ',' + this.props.cy + ')';

        const color = this.props.color;
        if (!color && this.props.claimed !== -1) {
            return (
                <g transform={transform}>
                  <polygon
                    points={this.points().join(' ')}
                    style={{
                        fill: '#555555',
                    }}
                    />
                </g>
            );
        }

        let penguin = null;
        let circles = null;
        if (color) {
            penguin = (<Penguin color={color} size={this.props.sideLength}/>);
        } else {
            circles = this.circles();
        }

        return (
            <g transform={transform} onClick={this._onClick.bind(this)}>
              <polygon
                points={this.points().join(' ')}
                style={{
                    stroke: 'blue',
                    fill: this.getFill(),
                }}
              />
              {circles}
              {penguin}
            </g>
        );
    }

    getFill() {
        if (this.props.highlighted) {
            return 'yellow';
        }

        if (this.props.possible) {
            return 'green';
        }

        return 'white';
    }

    circles() {
        const r = this.props.sideLength / 10;
        const color = '#888888';
        if (this.props.fish === 1) {
            return [
              <circle
                key={0}
                style={{fill: color}}
                r={r}
              />,
            ];
        } else if (this.props.fish === 2) {
            return [
              <circle
                key={0}
                style={{fill: color}}
                cx={-r * 2}
                r={r}
              />,
              <circle
                key={1}
                style={{fill: color}}
                cx={r * 2}
                r={r}
              />,
            ];
        } else if (this.props.fish === 3) {
            const y_offset = r * 2 * Math.sin(Math.PI / 3)
            return [
              <circle
                key={0}
                style={{fill: color}}
                cx={-r * 2}
                cy={y_offset}
                r={r}
              />,
              <circle
                key={1}
                style={{fill: color}}
                cx={r * 2}
                cy={y_offset}
                r={r}
               />,
               <circle
                key={2}
                style={{fill: color}}
                cx={0}
                cy={-y_offset}
                r={r}
              />,
            ];
        } else {
            throw new Error(this.props.fish + ' is not a valid fish amount');
        }
    }

    _onClick(): void {
        this.props.onClick(this.props._key);
    }

    points(): Array<Array<number>> {
      const points = [];
      for (let i = 0; i < 6; ++i) {
          points.push(this.corner(i));
      }

        return points;
    }

    corner(i: number): Array<number> {
        const side_length = this.props.sideLength;

        const angle_deg = 60 * i;
        const angle_rad = angle_deg * Math.PI / 180;
        return [
            side_length * Math.sin(angle_rad),
            side_length * Math.cos(angle_rad),
        ];
    }

    static width(side_length: number): number {
        const angle_rad_1 = Math.PI / 3;
        const angle_rad_5 = Math.PI * 5 / 3;

        return side_length * (Math.sin(angle_rad_1) - Math.sin(angle_rad_5));
    }
}

export default Hex;
