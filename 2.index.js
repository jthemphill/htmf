(window.webpackJsonp=window.webpackJsonp||[]).push([[2],[,function(e,t,n){"use strict";var r=this&&this.__createBinding||(Object.create?function(e,t,n,r){void 0===r&&(r=n),Object.defineProperty(e,r,{enumerable:!0,get:function(){return t[n]}})}:function(e,t,n,r){void 0===r&&(r=n),e[r]=t[n]}),i=this&&this.__setModuleDefault||(Object.create?function(e,t){Object.defineProperty(e,"default",{enumerable:!0,value:t})}:function(e,t){e.default=t}),l=this&&this.__importStar||function(e){if(e&&e.__esModule)return e;var t={};if(null!=e)for(var n in e)"default"!==n&&Object.hasOwnProperty.call(e,n)&&r(t,e,n);return i(t,e),t},s=this&&this.__importDefault||function(e){return e&&e.__esModule?e:{default:e}};Object.defineProperty(t,"__esModule",{value:!0});const o=s(n(3)),a=s(n(7)),c=l(n(2)),u=s(n(16)),f=["blue","red","orange","green"];function h(e){const t=[];for(let n=0;n<60;++n)t.push(e.num_fish(n));const n=[],r=[],i=[];for(let t=0;t<2;++t)n.push(e.score(t)),r.push([...e.penguins(t)]),i.push([...e.claimed(t)]);return{activePlayer:e.active_player(),modeType:e.finished_drafting()?"playing":"drafting",scores:n,turn:e.turn(),board:{fish:t,penguins:r,claimed:i}}}class d extends o.default.Component{constructor(e){super(e),this.handleCellClick=this._handleCellClick.bind(this),this.game=c.Game.new();const t=[];for(let e=0;e<30;++e)t.push(1);for(let e=30;e<50;++e)t.push(2);for(let e=50;e<60;++e)t.push(3);!function(e){for(let t=e.length;t;t--){const n=Math.floor(Math.random()*t),r=e[t-1];e[t-1]=e[n],e[n]=r}}(t),this.state={gameState:h(this.game),inputText:"",minDim:Math.min(window.innerWidth,window.innerHeight),chosenCell:null}}activePlayer(){return this.state.gameState.activePlayer}render(){const e=this.state.lastMoveInvalid?"Invalid move!":null,t=[],n=this.activePlayer();for(let e=0;e<2;++e){let r={color:f[e]},i=n===e?"(Active Player)":null;t.push(o.default.createElement("p",{key:"score_"+e},o.default.createElement("span",{style:r},"Score: ",this.state.gameState.scores[e]," ",i)))}return o.default.createElement("div",{className:"App",style:{display:"grid"}},o.default.createElement(u.default,{gameState:this.state.gameState,minDim:this.state.minDim,possibleMoves:(r=this.game,i=this.state.chosenCell,r.finished_drafting()?null==i?[]:[...r.possible_moves(i)]:[...r.draftable_cells()]),chosenCell:this.state.chosenCell,handleCellClick:this.handleCellClick}),o.default.createElement("div",{className:"info-col",style:{gridColumn:"12 / auto"}},o.default.createElement("p",null,this.state.gameState.modeType),o.default.createElement("p",null,e),o.default.createElement("div",null,t),o.default.createElement("input",{value:this.state.inputText,type:"text",onChange:e=>{this.setState({inputText:e.target.value})},onBlur:e=>{try{JSON.parse(this.state.inputText)}catch(e){return}}})));var r,i}componentDidMount(){window.addEventListener("resize",this.updateWindowDimensions.bind(this)),this.ponderer=window.setInterval(()=>{if(this.game.game_over())return;const e=performance.now();let t=0;for(;performance.now()-e<100;)this.game.playout(),++t;console.log(`${t} playouts in ${performance.now()-e} ms`)},1e3)}componentWillUnmount(){window.removeEventListener("resize",this.updateWindowDimensions.bind(this)),window.clearInterval(this.ponderer)}updateWindowDimensions(){this.setState({minDim:Math.min(window.innerWidth,window.innerHeight)})}_handleCellClick(e){const t=this.activePlayer();null!=t&&("drafting"!==this.state.gameState.modeType?this.state.gameState.board.penguins[t].includes(e)?this._toggleCellHighlight(e):null!=this.state.chosenCell&&this.game.possible_moves(this.state.chosenCell).includes(e)&&this._movePenguinToCell(e):this._placePenguin(e))}_placePenguin(e){let t=!1,n=this.state.chosenCell;try{this.game.place_penguin(e),n=null;const t=performance.now();for(;performance.now()-t<100;)this.game.playout();this.game.take_action()}catch(e){t=!0}this.setState({lastMoveInvalid:t,gameState:h(this.game),chosenCell:null})}_movePenguinToCell(e){let t=!1,n=this.state.chosenCell;try{this.game.move_penguin(this.state.chosenCell,e),n=null;const t=performance.now();for(;performance.now()-t<100;)this.game.playout();this.game.take_action()}catch(e){t=!0}this.setState({lastMoveInvalid:t,gameState:h(this.game),chosenCell:n})}_toggleCellHighlight(e){this.state.chosenCell!==e?this.setState({chosenCell:e}):this.setState({chosenCell:null})}}a.default.render(o.default.createElement(d,null),document.getElementById("root"))},function(e,t,n){"use strict";n.r(t),n.d(t,"Game",(function(){return b})),n.d(t,"__wbindgen_string_new",(function(){return v})),n.d(t,"__wbg_new_59cb74e423758ede",(function(){return w})),n.d(t,"__wbg_stack_558ba5917b466edd",(function(){return E})),n.d(t,"__wbg_error_4bb6c2a97407129a",(function(){return C})),n.d(t,"__wbindgen_object_drop_ref",(function(){return k})),n.d(t,"__wbg_getRandomValues_40ceff860009fa55",(function(){return M})),n.d(t,"__wbg_randomFillSync_eabbc18af655bfbe",(function(){return O})),n.d(t,"__wbg_self_e70540c4956ad879",(function(){return P})),n.d(t,"__wbg_require_9edeecb69c9dc31c",(function(){return x})),n.d(t,"__wbg_crypto_58b0c631995fea92",(function(){return j})),n.d(t,"__wbindgen_is_undefined",(function(){return S})),n.d(t,"__wbg_getRandomValues_532ec62d8e780edc",(function(){return D})),n.d(t,"__wbindgen_throw",(function(){return T})),n.d(t,"__wbindgen_rethrow",(function(){return z}));var r=n(14);let i=new("undefined"==typeof TextDecoder?n(5).TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});i.decode();let l=null;function s(){return null!==l&&l.buffer===r.v.buffer||(l=new Uint8Array(r.v.buffer)),l}function o(e,t){return i.decode(s().subarray(e,e+t))}const a=new Array(32);a.fill(void 0),a.push(void 0,null,!0,!1);let c=a.length;function u(e){c===a.length&&a.push(a.length+1);const t=c;return c=a[t],a[t]=e,t}function f(e){return a[e]}function h(e){const t=f(e);return function(e){e<36||(a[e]=c,c=e)}(e),t}let d=null;function p(){return null!==d&&d.buffer===r.v.buffer||(d=new Int32Array(r.v.buffer)),d}function m(e,t){return s().subarray(e/1,e/1+t)}let _=0;let g=new("undefined"==typeof TextEncoder?n(5).TextEncoder:TextEncoder)("utf-8");const y="function"==typeof g.encodeInto?function(e,t){return g.encodeInto(e,t)}:function(e,t){const n=g.encode(e);return t.set(n),{read:e.length,written:n.length}};class b{static __wrap(e){const t=Object.create(b.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,r.a(e)}static new(){var e=r.m();return b.__wrap(e)}is_drafting(){return 0!==r.k(this.ptr)}finished_drafting(){return 0!==r.i(this.ptr)}game_over(){return 0!==r.j(this.ptr)}active_player(){var e=r.f(this.ptr);return 16777215===e?void 0:e}score(e){return r.s(this.ptr,e)>>>0}turn(){return r.u(this.ptr)>>>0}num_fish(e){return r.n(this.ptr,e)>>>0}penguins(e){r.o(8,this.ptr,e);var t=p()[2],n=p()[3],i=m(t,n).slice();return r.c(t,1*n),i}claimed(e){r.g(8,this.ptr,e);var t=p()[2],n=p()[3],i=m(t,n).slice();return r.c(t,1*n),i}draftable_cells(){r.h(8,this.ptr);var e=p()[2],t=p()[3],n=m(e,t).slice();return r.c(e,1*t),n}possible_moves(e){r.r(8,this.ptr,e);var t=p()[2],n=p()[3],i=m(t,n).slice();return r.c(t,1*n),i}place_penguin(e){r.p(this.ptr,e)}move_penguin(e,t){r.l(this.ptr,e,t)}playout(){r.q(this.ptr)}take_action(){r.t(this.ptr)}}const v=function(e,t){return u(o(e,t))},w=function(){return u(new Error)},E=function(e,t){var n=function(e,t,n){if(void 0===n){const n=g.encode(e),r=t(n.length);return s().subarray(r,r+n.length).set(n),_=n.length,r}let r=e.length,i=t(r);const l=s();let o=0;for(;o<r;o++){const t=e.charCodeAt(o);if(t>127)break;l[i+o]=t}if(o!==r){0!==o&&(e=e.slice(o)),i=n(i,r,r=o+3*e.length);const t=s().subarray(i+o,i+r);o+=y(e,t).written}return _=o,i}(f(t).stack,r.d,r.e),i=_;p()[e/4+1]=i,p()[e/4+0]=n},C=function(e,t){try{console.error(o(e,t))}finally{r.c(e,t)}},k=function(e){h(e)},M=function(e,t,n){f(e).getRandomValues(m(t,n))},O=function(e,t,n){f(e).randomFillSync(m(t,n))},P=function(){try{return u(self.self)}catch(e){!function(e){r.b(u(e))}(e)}},x=function(e,t){return u(n(15)(o(e,t)))},j=function(e){return u(f(e).crypto)},S=function(e){return void 0===f(e)},D=function(e){return u(f(e).getRandomValues)},T=function(e,t){throw new Error(o(e,t))},z=function(e){throw h(e)}},,,,,,,,,,,,function(e,t,n){"use strict";var r=n.w[e.i];e.exports=r;n(2);r.w()},function(e,t){function n(e){var t=new Error("Cannot find module '"+e+"'");throw t.code="MODULE_NOT_FOUND",t}n.keys=function(){return[]},n.resolve=n,e.exports=n,n.id=15},function(e,t,n){"use strict";var r=this&&this.__createBinding||(Object.create?function(e,t,n,r){void 0===r&&(r=n),Object.defineProperty(e,r,{enumerable:!0,get:function(){return t[n]}})}:function(e,t,n,r){void 0===r&&(r=n),e[r]=t[n]}),i=this&&this.__setModuleDefault||(Object.create?function(e,t){Object.defineProperty(e,"default",{enumerable:!0,value:t})}:function(e,t){e.default=t}),l=this&&this.__importStar||function(e){if(e&&e.__esModule)return e;var t={};if(null!=e)for(var n in e)"default"!==n&&Object.hasOwnProperty.call(e,n)&&r(t,e,n);return i(t,e),t},s=this&&this.__importDefault||function(e){return e&&e.__esModule?e:{default:e}};Object.defineProperty(t,"__esModule",{value:!0});const o=l(n(3)),a=s(n(17));class c extends o.Component{render(){const e=this.props.minDim/16,t=2*e,n=2*e,r=["blue","red","orange","green"],i=[];for(let e=0;e<this.props.gameState.board.penguins.length;++e){let t=this.props.gameState.board.penguins[e];for(let n of t)i[n]=r[e]}const l=this.props.possibleMoves.length>0,s=new Set([]);for(let e of this.props.gameState.board.claimed)for(let t of e)s.add(t);const c=[];for(let r=0;r<8;++r){const u=n+1.5*r*e,f=r%2==0?7:8,h=r%2*-1*Math.sqrt(3)*e/2;for(let n=0;n<f;++n){const r=t+n*a.default.width(e)+h,f=c.length,d=this.props.gameState.board.fish[f],p=i[f],m=this.props.possibleMoves.includes(f),_=l&&this.props.chosenCell===f;c.push(o.createElement(a.default,{key:f,_key:f,onClick:this.props.handleCellClick,fish:d,cx:r,cy:u,sideLength:e,highlighted:_,possible:m,color:p,claimed:s.has(f)}))}}const u={height:this.props.minDim,width:this.props.minDim,gridColumn:"1 / auto"};return o.createElement("svg",{version:"1.1",style:u,baseProfile:"full",xmlns:"http://www.w3.org/2000/svg"},c)}}t.default=c},function(e,t,n){"use strict";var r=this&&this.__createBinding||(Object.create?function(e,t,n,r){void 0===r&&(r=n),Object.defineProperty(e,r,{enumerable:!0,get:function(){return t[n]}})}:function(e,t,n,r){void 0===r&&(r=n),e[r]=t[n]}),i=this&&this.__setModuleDefault||(Object.create?function(e,t){Object.defineProperty(e,"default",{enumerable:!0,value:t})}:function(e,t){e.default=t}),l=this&&this.__importStar||function(e){if(e&&e.__esModule)return e;var t={};if(null!=e)for(var n in e)"default"!==n&&Object.hasOwnProperty.call(e,n)&&r(t,e,n);return i(t,e),t},s=this&&this.__importDefault||function(e){return e&&e.__esModule?e:{default:e}};Object.defineProperty(t,"__esModule",{value:!0});const o=l(n(3)),a=s(n(18));class c extends o.PureComponent{render(){const e=`translate(${this.props.cx},${this.props.cy})`,t=this.props.color;if(!t&&this.props.claimed)return o.createElement("g",{transform:e},o.createElement("polygon",{points:this.points().join(" "),style:{fill:"#555555"}}));let n=null,r=null;return t?n=o.createElement(a.default,{color:t,size:this.props.sideLength}):r=this.circles(),o.createElement("g",{transform:e,onClick:this._onClick.bind(this)},o.createElement("polygon",{points:this.points().join(" "),style:{stroke:"blue",fill:this.getFill()}}),r,n)}getFill(){return this.props.highlighted?"#fffe9f":this.props.possible?"#d1fec8":"#eeeeee"}circles(){const e=this.props.sideLength/10,t="#888888";if(1===this.props.fish)return[o.createElement("circle",{key:0,style:{fill:t},r:e})];if(2===this.props.fish)return[o.createElement("circle",{key:0,style:{fill:t},cx:2*-e,r:e}),o.createElement("circle",{key:1,style:{fill:t},cx:2*e,r:e})];if(3===this.props.fish){const n=2*e*Math.sin(Math.PI/3);return[o.createElement("circle",{key:0,style:{fill:t},cx:2*-e,cy:n,r:e}),o.createElement("circle",{key:1,style:{fill:t},cx:2*e,cy:n,r:e}),o.createElement("circle",{key:2,style:{fill:t},cx:0,cy:-n,r:e})]}throw new Error(this.props.fish+" is not a valid fish amount")}_onClick(){this.props.onClick(this.props._key)}points(){const e=[];for(let t=0;t<6;++t)e.push(this.corner(t));return e}corner(e){const t=this.props.sideLength,n=e*Math.PI/3;return[t*Math.sin(n),t*Math.cos(n)]}static width(e){return e*(Math.sin(Math.PI/3)-Math.sin(5*Math.PI/3))}}t.default=c},function(e,t,n){"use strict";var r=this&&this.__createBinding||(Object.create?function(e,t,n,r){void 0===r&&(r=n),Object.defineProperty(e,r,{enumerable:!0,get:function(){return t[n]}})}:function(e,t,n,r){void 0===r&&(r=n),e[r]=t[n]}),i=this&&this.__setModuleDefault||(Object.create?function(e,t){Object.defineProperty(e,"default",{enumerable:!0,value:t})}:function(e,t){e.default=t}),l=this&&this.__importStar||function(e){if(e&&e.__esModule)return e;var t={};if(null!=e)for(var n in e)"default"!==n&&Object.hasOwnProperty.call(e,n)&&r(t,e,n);return i(t,e),t};Object.defineProperty(t,"__esModule",{value:!0});const s=l(n(3)),o=s.memo((function(e){const t={fill:e.color,fillOpacity:1,stroke:"none"},n=e.size/500;return s.createElement("g",{id:"layer1",transform:`scale(${n}),translate(-307.62496,-350)`},s.createElement("g",{id:"g3186",transform:"matrix(2.0165499,0,0,2.0165499,-378.20444,-374.28247)"},s.createElement("g",{transform:"translate(0,-14)",id:"g3176"},s.createElement("rect",{style:{fill:"#fb8b00",fillOpacity:1,stroke:"none"},id:"rect3164",width:"18.182745",height:"72.73098",x:"340.37064",y:"378.86526"}),s.createElement("path",{style:{fill:"#fb8b00",stroke:"none"},d:"m 347.12481,446.95339 -60.42857,21.85714 45.71428,18.57143 26.14286,-35.78572 z",id:"path3174"})),s.createElement("g",{transform:"matrix(-1,0,0,1,745.35258,-14)",id:"g3180"},s.createElement("rect",{y:"378.86526",x:"340.37064",height:"72.73098",width:"18.182745",id:"rect3182",style:{fill:"#fb8b00",fillOpacity:1,stroke:"none"}}),s.createElement("path",{id:"path3184",d:"m 347.12481,446.95339 -60.42857,21.85714 45.71428,18.57143 26.14286,-35.78572 z",style:{fill:"#fb8b00",stroke:"none"}})),s.createElement("path",{transform:"matrix(0.59003831,0,0,0.59003831,186.08628,108.73722)",d:"m 446.48743,416.47116 c 0,72.80489 -58.79389,131.82491 -131.31983,131.82491 -72.52593,0 -131.31982,-59.02002 -131.31982,-131.82491 0,-72.80488 58.79389,-131.8249 131.31982,-131.8249 72.52594,0 131.31983,59.02002 131.31983,131.8249 z",id:"path2985",style:t}),s.createElement("path",{transform:"translate(59.910095,0)",d:"m 361.63462,289.69702 c 0,27.3367 -22.16077,49.49748 -49.49747,49.49748 -27.3367,0 -49.49748,-22.16078 -49.49748,-49.49748 0,-27.3367 22.16078,-49.49747 49.49748,-49.49747 27.3367,0 49.49747,22.16077 49.49747,49.49747 z",id:"path2987",style:t}),s.createElement("path",{style:t,id:"path2989",d:"m 446.48743,416.47116 c 0,72.80489 -58.79389,131.82491 -131.31983,131.82491 -72.52593,0 -131.31982,-59.02002 -131.31982,-131.82491 0,-72.80488 58.79389,-131.8249 131.31982,-131.8249 72.52594,0 131.31983,59.02002 131.31983,131.8249 z",transform:"matrix(0.38675689,0,0,0.38675689,250.154,218.19562)"}),s.createElement("path",{id:"rect3147",d:"m 345.78328,275.11088 c 0,0 6.9297,-6 26.26396,-6 19.33426,0 26.26397,6 26.26397,6 l -26.26397,52.52793 z",style:{fill:"#fb8b00",fillOpacity:1,stroke:"none"}}),s.createElement("path",{transform:"translate(0,-8)",style:{fill:"#ffffff",fillOpacity:1,stroke:"none"},id:"path3152",d:"m 369.46331,266.71603 c 0,5.43945 -4.40955,9.84899 -9.84899,9.84899 -5.43945,0 -9.84899,-4.40954 -9.84899,-9.84899 0,-5.43944 4.40954,-9.84898 9.84899,-9.84898 5.43944,0 9.84899,4.40954 9.84899,9.84898 z"}),s.createElement("path",{transform:"translate(0,-8)",d:"m 391.93917,269.36768 c 0,3.55656 -2.88316,6.43972 -6.43972,6.43972 -3.55656,0 -6.43972,-2.88316 -6.43972,-6.43972 0,-3.55656 2.88316,-6.43973 6.43972,-6.43973 3.55656,0 6.43972,2.88317 6.43972,6.43973 z",id:"path3154",style:{fill:"#ffffff",fillOpacity:1,stroke:"none"}}),s.createElement("path",{id:"path3160",d:"M 332.14074,297.03867 247.29889,260.5461 c 10.94712,32.37883 32.49618,64.01962 75.63693,75.93783 z",style:t}),s.createElement("path",{style:t,d:"M 411.95375,297.03867 496.7956,260.5461 c -10.94712,32.37883 -32.49618,64.01962 -75.63693,75.93783 z",id:"path3162"})))}));t.default=o}]]);