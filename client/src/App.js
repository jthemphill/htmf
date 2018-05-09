// @flow

import React from 'react';

import Board from './Board.react';
import type GameState from './GameState'

import './App.css';

function shuffle<T>(a: Array<T>): void {
    for (let i = a.length; i; i--) {
        const j = Math.floor(Math.random() * i);
        const tmp = a[i - 1];
        a[i - 1] = a[j];
        a[j] = tmp;
    }
}

type State = {
  gameState: GameState,
  inputText: string,
};

type Props = {

};

class App extends React.Component<Props, State> {

    socket: WebSocket;

    constructor(props: Props) {
        super(props);

        const fish = [];
        const penguins = [[], []];
        const claimed = [];
        const possible_moves = [];

        for (let i = 0; i < 60; ++i) {
            if (i < 30) {
                fish[i] = 1;
            } else if (i < 50) {
                fish[i] = 2;
            } else {
                fish[i] = 3;
            }

            claimed[i] = -1;
            possible_moves[i] = false;
        }

        shuffle(fish);
        const gameState = {
            last_move_valid: true,
            mode_type: "drafting",
            nplayers: 2,
            board: {
                fish,
                penguins,
                claimed,
                possible_moves,
            },
            turn: 0,
            scores: [0, 0],
        };
        this.state = {
            gameState,
        };
    }

  activePlayer(): number {
      return this.state.gameState.active_player;
  }

  render() {
      const possible_moves = [];
      for (let x of this.state.gameState.board.possible_moves) {
          possible_moves.push(x);
      }
      const invalid_move_block = this.state.gameState.last_move_valid
            ? null
            : "Invalid move!";

      // duplicated in board.react.js
      const player_colors = ['blue', 'red', 'orange', 'green'];

      const scores_block = [];
      const active_player = this.activePlayer();
      for (let p = 0; p < this.state.gameState.nplayers; ++p) {
          let color = {color: player_colors[p]};
          let active = active_player === p ? '(Active Player)' : null;
          scores_block.push(
            <p key={"score_"+p}><span style={color}>
              Score: {this.state.gameState.scores[p]} {active}
            </span></p>
          );
      }

      return (
        <div className="App" style={{'display': 'grid'}}>
          <Board
            socket={this.socket}
            gameState={this.state.gameState}
            possibleMoves={possible_moves}
          />
            <div className="info-col" style={{'gridColumn': '12 / auto'}}>
            <p>{this.state.gameState.mode_type}</p>
            <p>{invalid_move_block}</p>
            <div>{scores_block}</div>
            <input value={this.state.inputText}
                   type="text"
                   onChange={(e) => {
                       this.setState({inputText: e.target.value});
                   }}
                   onBlur={(e) => {
                     try {
                       const input_data = JSON.parse(this.state.inputText);
                       this.socket.send(JSON.stringify({
                         "action_type": "setup",
                         "data": {state: input_data},
                       }));
                     } catch (_) {
                       return;
                     }
                 }} />
          </div>
        </div>
      );
  }

    componentDidMount() {
        this.socket = new WebSocket('ws://127.0.0.1:2794');
        this.socket.onmessage = (e: MessageEvent) => {
            if (typeof e.data !== 'string') {
                throw TypeError(
                    [
                        'TypeError:',
                        JSON.stringify(e.data),
                        'is not a string',
                    ].join(' ')
                );
            }
            const new_state = JSON.parse(e.data);
            console.log('Received state: ', new_state);
            this.setState({gameState: new_state, inputText: JSON.stringify(new_state)});
        };
    }

    componentWillUnmount() {
        this.socket.close();
    }
}

export default App;
