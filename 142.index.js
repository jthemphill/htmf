(globalThis.webpackChunkhtmf=globalThis.webpackChunkhtmf||[]).push([[142],{942:(e,t,n)=>{"use strict";n.a(e,(async r=>{n.d(t,{lA:()=>P,h4:()=>M,h9:()=>S,Dz:()=>T,kF:()=>O,ug:()=>j,cx:()=>A,C2:()=>E,rY:()=>I,Wl:()=>x,UE:()=>C,Im:()=>F,eY:()=>D,dS:()=>U,Oi:()=>Y,gl:()=>B,tg:()=>z,rz:()=>J,m_:()=>q,T3:()=>G,XY:()=>X,ig:()=>H,zK:()=>K,XP:()=>N,Bn:()=>Q,JF:()=>R,BS:()=>V,i7:()=>W,E_:()=>Z,FC:()=>L,Or:()=>$,nD:()=>ee,oH:()=>te});var s=n(335);e=n.hmd(e);var i=r([s]);s=(i.then?await i:i)[0];let o=new("undefined"==typeof TextDecoder?(0,e.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});o.decode();let a=null;function _(){return null!==a&&a.buffer===s.memory.buffer||(a=new Uint8Array(s.memory.buffer)),a}function c(e,t){return o.decode(_().subarray(e,e+t))}const u=new Array(32).fill(void 0);u.push(void 0,null,!0,!1);let l=u.length;function g(e){l===u.length&&u.push(u.length+1);const t=l;return l=u[t],u[t]=e,t}function d(e){return u[e]}function f(e){const t=d(e);return function(e){e<36||(u[e]=l,l=e)}(e),t}let p=null;function h(){return null!==p&&p.buffer===s.memory.buffer||(p=new Int32Array(s.memory.buffer)),p}function b(e,t){return _().subarray(e/1,e/1+t)}let m=0,w=new("undefined"==typeof TextEncoder?(0,e.require)("util").TextEncoder:TextEncoder)("utf-8");const v="function"==typeof w.encodeInto?function(e,t){return w.encodeInto(e,t)}:function(e,t){const n=w.encode(e);return t.set(n),{read:e.length,written:n.length}};function y(e,t){try{return e.apply(this,t)}catch(e){s.__wbindgen_exn_store(g(e))}}class P{static __wrap(e){const t=Object.create(P.prototype);return t.ptr=e,t}__destroy_into_raw(){const e=this.ptr;return this.ptr=0,e}free(){const e=this.__destroy_into_raw();s.__wbg_game_free(e)}static new(){var e=s.game_new();return P.__wrap(e)}is_drafting(){return 0!==s.game_is_drafting(this.ptr)}finished_drafting(){return 0!==s.game_finished_drafting(this.ptr)}game_over(){return 0!==s.game_game_over(this.ptr)}active_player(){var e=s.game_active_player(this.ptr);return 16777215===e?void 0:e}score(e){return s.game_score(this.ptr,e)>>>0}turn(){return s.game_turn(this.ptr)>>>0}num_fish(e){return s.game_num_fish(this.ptr,e)>>>0}penguins(e){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.game_penguins(i,this.ptr,e);var t=h()[i/4+0],n=h()[i/4+1],r=b(t,n).slice();return s.__wbindgen_free(t,1*n),r}finally{s.__wbindgen_add_to_stack_pointer(16)}}claimed(e){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.game_claimed(i,this.ptr,e);var t=h()[i/4+0],n=h()[i/4+1],r=b(t,n).slice();return s.__wbindgen_free(t,1*n),r}finally{s.__wbindgen_add_to_stack_pointer(16)}}draftable_cells(){try{const r=s.__wbindgen_add_to_stack_pointer(-16);s.game_draftable_cells(r,this.ptr);var e=h()[r/4+0],t=h()[r/4+1],n=b(e,t).slice();return s.__wbindgen_free(e,1*t),n}finally{s.__wbindgen_add_to_stack_pointer(16)}}possible_moves(e){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.game_possible_moves(i,this.ptr,e);var t=h()[i/4+0],n=h()[i/4+1],r=b(t,n).slice();return s.__wbindgen_free(t,1*n),r}finally{s.__wbindgen_add_to_stack_pointer(16)}}place_penguin(e){s.game_place_penguin(this.ptr,e)}move_penguin(e,t){s.game_move_penguin(this.ptr,e,t)}playout(){s.game_playout(this.ptr)}get_visits(){return s.game_get_visits(this.ptr)}place_info(e){var t=s.game_place_info(this.ptr,e);return k.__wrap(t)}move_info(e,t){var n=s.game_move_info(this.ptr,e,t);return k.__wrap(n)}take_action(){s.game_take_action(this.ptr)}}class k{static __wrap(e){const t=Object.create(k.prototype);return t.ptr=e,t}__destroy_into_raw(){const e=this.ptr;return this.ptr=0,e}free(){const e=this.__destroy_into_raw();s.__wbg_moveinfo_free(e)}get_visits(){return s.moveinfo_get_visits(this.ptr)}get_rewards(){return s.moveinfo_get_rewards(this.ptr)}}function M(e,t){return g(c(e,t))}function S(){return g(new Error)}function T(e,t){var n=function(e,t,n){if(void 0===n){const n=w.encode(e),r=t(n.length);return _().subarray(r,r+n.length).set(n),m=n.length,r}let r=e.length,s=t(r);const i=_();let o=0;for(;o<r;o++){const t=e.charCodeAt(o);if(t>127)break;i[s+o]=t}if(o!==r){0!==o&&(e=e.slice(o)),s=n(s,r,r=o+3*e.length);const t=_().subarray(s+o,s+r);o+=v(e,t).written}return m=o,s}(d(t).stack,s.__wbindgen_malloc,s.__wbindgen_realloc),r=m;h()[e/4+1]=r,h()[e/4+0]=n}function O(e,t){try{console.error(c(e,t))}finally{s.__wbindgen_free(e,t)}}function j(e){f(e)}function A(){return y((function(e,t,n){d(e).randomFillSync(b(t,n))}),arguments)}function E(){return y((function(e,t){d(e).getRandomValues(d(t))}),arguments)}function I(e){return g(d(e).process)}function x(e){const t=d(e);return"object"==typeof t&&null!==t}function C(e){return g(d(e).versions)}function F(e){return g(d(e).node)}function D(e){return"string"==typeof d(e)}function U(){return y((function(e,t){return g(n(989)(c(e,t)))}),arguments)}function Y(e){return g(d(e).crypto)}function B(e){return g(d(e).msCrypto)}function z(e,t){return g(new Function(c(e,t)))}function J(){return y((function(e,t){return g(d(e).call(d(t)))}),arguments)}function q(e){return g(d(e))}function G(){return y((function(){return g(self.self)}),arguments)}function X(){return y((function(){return g(window.window)}),arguments)}function H(){return y((function(){return g(globalThis.globalThis)}),arguments)}function K(){return y((function(){return g(n.g.global)}),arguments)}function N(e){return void 0===d(e)}function Q(e){return g(d(e).buffer)}function R(e){return g(new Uint8Array(d(e)))}function V(e,t,n){d(e).set(d(t),n>>>0)}function W(e){return d(e).length}function Z(e){return g(new Uint8Array(e>>>0))}function L(e,t,n){return g(d(e).subarray(t>>>0,n>>>0))}function $(e,t){throw new Error(c(e,t))}function ee(e){throw f(e)}function te(){return g(s.memory)}}))},989:e=>{function t(e){var t=new Error("Cannot find module '"+e+"'");throw t.code="MODULE_NOT_FOUND",t}t.keys=()=>[],t.resolve=t,t.id=989,e.exports=t},142:(e,t,n)=>{"use strict";n.a(e,(async e=>{n.r(t),n.d(t,{default:()=>o});var r=n(942),s=n(601),i=e([r]);r=(i.then?await i:i)[0];const o=class{constructor(e){Object.defineProperty(this,"game",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"postMessage",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"ponderer",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"nplayouts",{enumerable:!0,configurable:!0,writable:!0,value:0}),this.game=r.lA.new(),this.postMessage=e,this.ponder()}free(){this.stopPondering(),this.game.free()}init(){const e=this.postMessage;e({type:"state",gameState:this.getState()}),e({type:"possibleMoves",possibleMoves:this.getPossibleMoves()})}ponder(){this.nplayouts=this.game.get_visits(),void 0===this.ponderer&&(this.ponderer=self.setInterval((()=>{if(this.nplayouts>=s.BS)return void this.stopPondering();const e=performance.now();for(;performance.now()-e<s._U;)this.playout();let t=this.game.active_player();void 0!==t&&(this.game.is_drafting()?this.postPlaceScores(t):this.postMoveScores(t))}),s.c9))}stopPondering(){void 0!==this.ponderer&&(self.clearInterval(this.ponderer),this.ponderer=void 0)}placePenguin(e){this.game.place_penguin(e),this.ponder()}movePenguin(e,t){this.game.move_penguin(e,t),this.ponder()}playout(){this.game.playout(),++this.nplayouts}takeAction(){this.stopPondering();const e=this.postMessage,t=this.game.turn()<2?2*s.YP:s.YP;for(;this.nplayouts<t;)this.playout(),this.nplayouts%100==0&&e({type:"thinkingProgress",completed:this.nplayouts,required:t});this.game.take_action(),this.ponder()}getState(){return function(e){const t=[];for(let n=0;n<s.JQ;++n)t.push(e.num_fish(n));const n=[],r=[],i=[];for(let t=0;t<s.Zp;++t)n.push(e.score(t)),r.push([...e.penguins(t)]),i.push([...e.claimed(t)]);return{activePlayer:e.active_player(),modeType:e.finished_drafting()?"playing":"drafting",scores:n,turn:e.turn(),board:{fish:t,penguins:r,claimed:i}}}(this.game)}getPossibleMoves(e){return function(e,t){if(e.active_player()!=s.Jh)return[];if(e.finished_drafting()){if(void 0!==t)return[...e.possible_moves(t)];const n=e.active_player();return void 0===n?[]:[...e.penguins(n)]}return[...e.draftable_cells()]}(this.game,e)}onMessage(e){switch(console.log(`received request ${e.type}`),e.type){case"get":this.postGameState();break;case"move":try{this.movePenguin(e.src,e.dst),this.postGameState()}catch(t){this.postIllegalMove(e.src,e.dst)}break;case"place":try{this.placePenguin(e.dst),this.postGameState()}catch(t){this.postIllegalPlacement(e.dst)}break;case"possibleMoves":this.postPossibleMoves(e.src);break;case"takeAction":this.takeAction(),this.postGameState()}}postIllegalMove(e,t){(0,this.postMessage)({type:"illegalMove",src:e,dst:t})}postIllegalPlacement(e){(0,this.postMessage)({type:"illegalPlacement",dst:e})}postPossibleMoves(e){(0,this.postMessage)({type:"possibleMoves",possibleMoves:this.getPossibleMoves(e)})}postGameState(){(0,this.postMessage)({type:"state",gameState:this.getState()})}postPlaceScores(e){const t=[];for(let e of this.game.draftable_cells()){const n=this.game.place_info(e);t.push({dst:e,visits:n.get_visits(),rewards:n.get_rewards()})}(0,this.postMessage)({type:"placeScores",activePlayer:e,placeScores:t})}postMoveScores(e){const t=[];for(let n of this.game.penguins(e))for(let e of this.game.possible_moves(n)){const r=this.game.move_info(n,e);t.push({src:n,dst:e,visits:r.get_visits(),rewards:r.get_rewards()})}(0,this.postMessage)({type:"moveScores",activePlayer:e,moveScores:t})}}}))},601:(e,t,n)=>{"use strict";n.d(t,{Zp:()=>r,Jh:()=>s,JQ:()=>i,_U:()=>o,YP:()=>_,BS:()=>c,c9:()=>a});const r=2,s=0,i=60,o=100,a=0,_=14e3,c=6e4},335:(e,t,n)=>{"use strict";var r=([r])=>n.v(t,e.id,"e4a125d57d64f925096d",{"./htmf_wasm_bg.js":{__wbindgen_string_new:r.h4,__wbg_new_59cb74e423758ede:r.h9,__wbg_stack_558ba5917b466edd:r.Dz,__wbg_error_4bb6c2a97407129a:r.kF,__wbindgen_object_drop_ref:r.ug,__wbg_randomFillSync_64cc7d048f228ca8:r.cx,__wbg_getRandomValues_98117e9a7e993920:r.C2,__wbg_process_2f24d6544ea7b200:r.rY,__wbindgen_is_object:r.Wl,__wbg_versions_6164651e75405d4a:r.UE,__wbg_node_4b517d861cbcb3bc:r.Im,__wbindgen_is_string:r.eY,__wbg_modulerequire_3440a4bcf44437db:r.dS,__wbg_crypto_98fc271021c7d2ad:r.Oi,__wbg_msCrypto_a2cdb043d2bfe57f:r.gl,__wbg_newnoargs_ac91a24e57fcaec8:r.tg,__wbg_call_9e1eb05d905a21d9:r.rz,__wbindgen_object_clone_ref:r.m_,__wbg_self_bce917bbd61b0be0:r.T3,__wbg_window_08048ce184ae3496:r.XY,__wbg_globalThis_d6f1ff349571af81:r.ig,__wbg_global_63b22b64d239db75:r.zK,__wbindgen_is_undefined:r.XP,__wbg_buffer_fbad716641c158a5:r.Bn,__wbg_new_c9e78bd69716df92:r.JF,__wbg_set_2fd4486048716f38:r.BS,__wbg_length_82dd1e63e9c75f09:r.i7,__wbg_newwithlength_a9f6c1fd1bf4e5e4:r.E_,__wbg_subarray_e80c85d931be89c4:r.FC,__wbindgen_throw:r.Or,__wbindgen_rethrow:r.nD,__wbindgen_memory:r.oH}});n.a(e,(e=>{var t=e([n(942)]);return t.then?t.then(r):r(t)}),1)}}]);