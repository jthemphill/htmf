import * as React from 'react'

import Penguin from './Penguin'

interface Props {
  _key: number
  onClick: (key: number) => void

  claimed: boolean
  fish: number
  cx: number
  cy: number
  sideLength: number
  highlighted: boolean
  possible: boolean
  player?: number
}

function points (props: Props): number[][] {
  const points = []
  for (let i = 0; i < 6; ++i) {
    points.push(corner(props, i))
  }
  return points
}

function corner (props: Props, i: number): number[] {
  const sideLength = props.sideLength
  const angleRadians = i * Math.PI / 3
  return [
    sideLength * Math.sin(angleRadians),
    sideLength * Math.cos(angleRadians)
  ]
}

function drawCircles (props: Props): React.JSX.Element[] {
  const r = props.sideLength / 10
  if (props.fish === 1) {
    return [
      <circle
        key={0}
        className="fish"
        r={r}
      />
    ]
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
      />
    ]
  } else if (props.fish === 3) {
    const yOffset = r * 2 * Math.sin(Math.PI / 3)
    return [
      <circle
        key={0}
        className="fish"
        cx={-r * 2}
        cy={yOffset}
        r={r}
      />,
      <circle
        key={1}
        className="fish"
        cx={r * 2}
        cy={yOffset}
        r={r}
      />,
      <circle
        key={2}
        className="fish"
        cx={0}
        cy={-yOffset}
        r={r}
      />
    ]
  } else {
    throw new Error(`${props.fish} is not a valid fish amount`)
  }
}

const Hex = React.memo(function (props: Props) {
  const transform = `translate(${props.cx},${props.cy})`

  const player = props.player
  if (player === undefined && props.claimed) {
    return (
      <g transform={transform}>
        <polygon
          points={points(props).join(' ')}
          className="cell claimed"
        />
      </g>
    )
  }

  let penguin
  let circles
  if (player !== undefined) {
    penguin = (<Penguin player={player} size={props.sideLength} />)
  } else {
    circles = drawCircles(props)
  }
  const cellClasses = ['cell']
  if (props.highlighted) {
    cellClasses.push('highlighted')
  } else if (props.possible) {
    cellClasses.push('possible')
  }
  return (
    <g transform={transform} onClick={() => { props.onClick(props._key) }}>
      <polygon
        className={cellClasses.join(' ')}
        points={points(props).join(' ')}
      />
      {circles}
      {penguin}
    </g>
  )
})

export default Hex
