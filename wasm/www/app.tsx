import React from "react";
import ReactDOM from "react-dom";
import * as wasm from "htmf-wasm";

import Board from "./Board";
import GameState from "./GameState";
import { NPLAYERS, NUM_CELLS, PLAYER_COLORS, PONDER_INTERVAL_MS, PLAYOUT_MS } from "./constants";

function getGameState(game: wasm.Game): GameState {
    const fish = [];
    for (let idx = 0; idx < NUM_CELLS; ++idx) {
        fish.push(game.num_fish(idx));
    }
    const scores: number[] = [];
    const penguins = [];
    const claimed = [];
    for (let p = 0; p < NPLAYERS; ++p) {
        scores.push(game.score(p));
        penguins.push([...game.penguins(p)]);
        claimed.push([...game.claimed(p)]);
    }
    return {
        activePlayer: game.active_player(),
        modeType: game.finished_drafting() ? "playing" : "drafting",
        scores,
        turn: game.turn(),
        board: {
            fish,
            penguins,
            claimed,
        }
    };
}

function getPossibleMoves(game: wasm.Game, chosenCell?: number): number[] {
    if (game.finished_drafting()) {
        return chosenCell == null ? [] : [...game.possible_moves(chosenCell)];
    } else {
        return [...game.draftable_cells()];
    }
}

type State = {
    gameState: GameState,
    chosenCell?: number,
    lastMoveInvalid?: boolean,
};
type Props = {

};
class App extends React.Component<Props, State> {

    handleCellClick: ((idx: number) => void);
    state: State;
    game: wasm.Game;
    ponderer?: number;

    constructor(props: Props) {
        super(props);
        this.handleCellClick = this._handleCellClick.bind(this);
        this.game = wasm.Game.new();
        this.state = {
            gameState: getGameState(this.game),
            chosenCell: null,
        };
    }

    activePlayer(): number {
        return this.state.gameState.activePlayer;
    }

    render() {
        const invalid_move_block = this.state.lastMoveInvalid
            ? "Invalid move!"
            : null;

        const scores_block = [];
        const active_player = this.activePlayer();
        for (let p = 0; p < NPLAYERS; ++p) {
            let color = { color: PLAYER_COLORS[p] };
            let active = active_player === p ? '(Active Player)' : null;
            scores_block.push(
                <p key={"score_" + p}><span style={color}>
                    Score: {this.state.gameState.scores[p]} {active}
                </span></p>
            );
        }

        return (
            <div className="app">
                <Board
                    gameState={this.state.gameState}
                    possibleMoves={getPossibleMoves(this.game, this.state.chosenCell)}
                    chosenCell={this.state.chosenCell}
                    handleCellClick={this.handleCellClick}
                />
                <div className="info-col">
                    <p>{this.state.gameState.modeType}</p>
                    <p>{invalid_move_block}</p>
                    <div>{scores_block}</div>
                </div>
            </div>
        );
    }

    componentDidMount() {
        this.ponderer = window.setInterval(
            () => {
                if (this.game.game_over()) {
                    return;
                }
                const t0 = performance.now();
                let nplayouts = 0;
                while (performance.now() - t0 < PLAYOUT_MS) {
                    this.game.playout();
                    ++nplayouts;
                }
                console.log(`${nplayouts} playouts in ${performance.now() - t0} ms`);
            }, PONDER_INTERVAL_MS
        );
    }

    componentWillUnmount() {
        window.clearInterval(this.ponderer);
        this.game.free();
        this.game = null;
    }

    _handleCellClick(key: number) {
        const activePlayer = this.activePlayer();
        if (activePlayer == null) {
            return;
        }
        if (this.state.gameState.modeType === "drafting") {
            this._placePenguin(key);
            return;
        }
        if (this.state.gameState.board.penguins[activePlayer].includes(key)) {
            this._toggleCellHighlight(key);
            return;
        }
        if (this.state.chosenCell == null) {
            return;
        }
        if (this.game.possible_moves(this.state.chosenCell).includes(key)) {
            this._movePenguinToCell(key);
            return;
        }
    }

    _placePenguin(key: number) {
        let lastMoveInvalid = false
        let chosenCell = this.state.chosenCell;
        try {
            this.game.place_penguin(key);
            chosenCell = null;
            const t0 = performance.now();
            while (performance.now() - t0 < PLAYOUT_MS) {
                this.game.playout();
            }
            this.game.take_action();
        } catch (e) {
            lastMoveInvalid = true;
        }
        this.setState({ lastMoveInvalid, gameState: getGameState(this.game), chosenCell: null });
    }

    _movePenguinToCell(key: number) {
        let lastMoveInvalid = false;
        let chosenCell = this.state.chosenCell;
        try {
            this.game.move_penguin(this.state.chosenCell, key);
            chosenCell = null;
            const t0 = performance.now();
            while (performance.now() - t0 < PLAYOUT_MS) {
                this.game.playout();
            }
            this.game.take_action();
        } catch (e) {
            lastMoveInvalid = true;
        }
        this.setState({ lastMoveInvalid, gameState: getGameState(this.game), chosenCell });
    }

    _toggleCellHighlight(key: number) {
        if (this.state.chosenCell === key) {
            this.setState({
                chosenCell: null,
            });
            return;
        }

        this.setState({
            chosenCell: key,
        });
    }
}

ReactDOM.render(<App />, document.getElementById('root'));