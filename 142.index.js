(globalThis.webpackChunkhtmf=globalThis.webpackChunkhtmf||[]).push([[142],{942:(e,t,n)=>{"use strict";n.a(e,(async(r,s)=>{try{n.d(t,{A7:()=>K,BT:()=>Q,GS:()=>E,GX:()=>C,Ih:()=>A,JX:()=>R,MF:()=>F,Or:()=>ne,PT:()=>V,Wl:()=>q,XP:()=>W,Y8:()=>D,YQ:()=>z,_8:()=>x,b0:()=>ee,cF:()=>J,cR:()=>Y,eY:()=>U,gk:()=>G,h4:()=>j,jp:()=>L,lA:()=>S,m_:()=>Z,nD:()=>re,oH:()=>se,ud:()=>N,uf:()=>te,ug:()=>I,vG:()=>X,wg:()=>B,xd:()=>H,y4:()=>$,yq:()=>O});var i=n(335);e=n.hmd(e);var o=r([i]);i=(o.then?(await o)():o)[0];let a=new("undefined"==typeof TextDecoder?(0,e.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});a.decode();let _=null;function c(){return null!==_&&_.buffer===i.memory.buffer||(_=new Uint8Array(i.memory.buffer)),_}function u(e,t){return a.decode(c().subarray(e,e+t))}const l=new Array(32).fill(void 0);l.push(void 0,null,!0,!1);let f=l.length;function g(e){f===l.length&&l.push(l.length+1);const t=f;return f=l[t],l[t]=e,t}function d(e){return l[e]}function p(e){e<36||(l[e]=f,f=e)}function h(e){const t=d(e);return p(e),t}let b=null;function w(){return null!==b&&b.buffer===i.memory.buffer||(b=new Int32Array(i.memory.buffer)),b}function m(e,t){return c().subarray(e/1,e/1+t)}let y=0,v=new("undefined"==typeof TextEncoder?(0,e.require)("util").TextEncoder:TextEncoder)("utf-8");const P="function"==typeof v.encodeInto?function(e,t){return v.encodeInto(e,t)}:function(e,t){const n=v.encode(e);return t.set(n),{read:e.length,written:n.length}};function M(e,t,n){if(void 0===n){const n=v.encode(e),r=t(n.length);return c().subarray(r,r+n.length).set(n),y=n.length,r}let r=e.length,s=t(r);const i=c();let o=0;for(;o<r;o++){const t=e.charCodeAt(o);if(t>127)break;i[s+o]=t}if(o!==r){0!==o&&(e=e.slice(o)),s=n(s,r,r=o+3*e.length);const t=c().subarray(s+o,s+r);o+=P(e,t).written}return y=o,s}function k(e,t){try{return e.apply(this,t)}catch(e){i.__wbindgen_exn_store(g(e))}}class S{static __wrap(e){const t=Object.create(S.prototype);return t.ptr=e,t}__destroy_into_raw(){const e=this.ptr;return this.ptr=0,e}free(){const e=this.__destroy_into_raw();i.__wbg_game_free(e)}static new(){var e=i.game_new();return S.__wrap(e)}is_drafting(){return 0!==i.game_is_drafting(this.ptr)}finished_drafting(){return 0!==i.game_finished_drafting(this.ptr)}game_over(){return 0!==i.game_game_over(this.ptr)}active_player(){var e=i.game_active_player(this.ptr);return 16777215===e?void 0:e}score(e){return i.game_score(this.ptr,e)>>>0}turn(){return i.game_turn(this.ptr)>>>0}num_fish(e){return i.game_num_fish(this.ptr,e)>>>0}penguins(e){try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.game_penguins(s,this.ptr,e);var t=w()[s/4+0],n=w()[s/4+1],r=m(t,n).slice();return i.__wbindgen_free(t,1*n),r}finally{i.__wbindgen_add_to_stack_pointer(16)}}claimed(e){try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.game_claimed(s,this.ptr,e);var t=w()[s/4+0],n=w()[s/4+1],r=m(t,n).slice();return i.__wbindgen_free(t,1*n),r}finally{i.__wbindgen_add_to_stack_pointer(16)}}draftable_cells(){try{const r=i.__wbindgen_add_to_stack_pointer(-16);i.game_draftable_cells(r,this.ptr);var e=w()[r/4+0],t=w()[r/4+1],n=m(e,t).slice();return i.__wbindgen_free(e,1*t),n}finally{i.__wbindgen_add_to_stack_pointer(16)}}possible_moves(e){try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.game_possible_moves(s,this.ptr,e);var t=w()[s/4+0],n=w()[s/4+1],r=m(t,n).slice();return i.__wbindgen_free(t,1*n),r}finally{i.__wbindgen_add_to_stack_pointer(16)}}place_penguin(e){i.game_place_penguin(this.ptr,e)}move_penguin(e,t){i.game_move_penguin(this.ptr,e,t)}playout(){i.game_playout(this.ptr)}get_visits(){return i.game_get_visits(this.ptr)}place_info(e){var t=i.game_place_info(this.ptr,e);return T.__wrap(t)}move_info(e,t){var n=i.game_move_info(this.ptr,e,t);return T.__wrap(n)}take_action(){i.game_take_action(this.ptr)}}class T{static __wrap(e){const t=Object.create(T.prototype);return t.ptr=e,t}__destroy_into_raw(){const e=this.ptr;return this.ptr=0,e}free(){const e=this.__destroy_into_raw();i.__wbg_moveinfo_free(e)}get_visits(){return i.moveinfo_get_visits(this.ptr)}get_rewards(){return i.moveinfo_get_rewards(this.ptr)}}function j(e,t){return g(u(e,t))}function A(){return g(new Error)}function O(e,t){var n=M(d(t).stack,i.__wbindgen_malloc,i.__wbindgen_realloc),r=y;w()[e/4+1]=r,w()[e/4+0]=n}function G(e,t){try{console.error(u(e,t))}finally{i.__wbindgen_free(e,t)}}function I(e){h(e)}function x(){return k((function(e,t,n){d(e).randomFillSync(m(t,n))}),arguments)}function Y(){return k((function(e,t){d(e).getRandomValues(d(t))}),arguments)}function F(e){return g(d(e).process)}function q(e){const t=d(e);return"object"==typeof t&&null!==t}function D(e){return g(d(e).versions)}function E(e){return g(d(e).node)}function U(e){return"string"==typeof d(e)}function C(){return k((function(e,t){return g(n(989)(u(e,t)))}),arguments)}function J(e){return g(d(e).crypto)}function X(e){return g(d(e).msCrypto)}function B(e,t){return g(new Function(u(e,t)))}function Q(){return k((function(e,t){return g(d(e).call(d(t)))}),arguments)}function R(){return k((function(){return g(self.self)}),arguments)}function H(){return k((function(){return g(window.window)}),arguments)}function N(){return k((function(){return g(globalThis.globalThis)}),arguments)}function V(){return k((function(){return g(n.g.global)}),arguments)}function W(e){return void 0===d(e)}function Z(e){return g(d(e))}function L(e){return g(d(e).buffer)}function $(e){return g(new Uint8Array(d(e)))}function z(e,t,n){d(e).set(d(t),n>>>0)}function K(e){return d(e).length}function ee(e){return g(new Uint8Array(e>>>0))}function te(e,t,n){return g(d(e).subarray(t>>>0,n>>>0))}function ne(e,t){throw new Error(u(e,t))}function re(e){throw h(e)}function se(){return g(i.memory)}s()}catch(ie){s(ie)}}))},989:e=>{function t(e){var t=new Error("Cannot find module '"+e+"'");throw t.code="MODULE_NOT_FOUND",t}t.keys=()=>[],t.resolve=t,t.id=989,e.exports=t},142:(e,t,n)=>{"use strict";n.a(e,(async(e,r)=>{try{n.r(t),n.d(t,{default:()=>u});var s=n(942),i=n(601),o=e([s]);function a(e){const t=[];for(let n=0;n<i.JQ;++n)t.push(e.num_fish(n));const n=[],r=[],s=[];for(let t=0;t<i.Zp;++t)n.push(e.score(t)),r.push([...e.penguins(t)]),s.push([...e.claimed(t)]);return{activePlayer:e.active_player(),modeType:e.finished_drafting()?"playing":"drafting",scores:n,turn:e.turn(),board:{fish:t,penguins:r,claimed:s}}}function _(e,t){if(e.active_player()!=i.Jh)return[];if(e.finished_drafting()){if(void 0!==t)return[...e.possible_moves(t)];const n=e.active_player();return void 0===n?[]:[...e.penguins(n)]}return[...e.draftable_cells()]}s=(o.then?(await o)():o)[0];class c{constructor(e){Object.defineProperty(this,"game",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"postMessage",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"ponderer",{enumerable:!0,configurable:!0,writable:!0,value:void 0}),Object.defineProperty(this,"nplayouts",{enumerable:!0,configurable:!0,writable:!0,value:0}),this.game=s.lA.new(),this.postMessage=e,this.ponder()}free(){this.stopPondering(),this.game.free()}init(){const e=this.postMessage;e({type:"state",gameState:this.getState()}),e({type:"possibleMoves",possibleMoves:this.getPossibleMoves()})}ponder(){this.nplayouts=this.game.get_visits(),void 0===this.ponderer&&(this.ponderer=self.setInterval((()=>{if(this.nplayouts>=i.BS)return void this.stopPondering();const e=performance.now();for(;performance.now()-e<i._U;)this.playout();let t=this.game.active_player();void 0!==t&&(this.game.is_drafting()?this.postPlaceScores(t):this.postMoveScores(t))}),i.c9))}stopPondering(){void 0!==this.ponderer&&(self.clearInterval(this.ponderer),this.ponderer=void 0)}placePenguin(e){this.game.place_penguin(e),this.ponder()}movePenguin(e,t){this.game.move_penguin(e,t),this.ponder()}playout(){this.game.playout(),++this.nplayouts}takeAction(){this.stopPondering();const e=this.postMessage,t=this.game.turn()<2?2*i.YP:i.YP;for(;this.nplayouts<t;)this.playout(),this.nplayouts%100==0&&e({type:"thinkingProgress",completed:this.nplayouts,required:t});this.game.take_action(),this.ponder()}getState(){return a(this.game)}getPossibleMoves(e){return _(this.game,e)}onMessage(e){switch(console.log(`received request ${e.type}`),e.type){case"get":this.postGameState();break;case"move":try{this.movePenguin(e.src,e.dst),this.postGameState()}catch(t){this.postIllegalMove(e.src,e.dst)}break;case"place":try{this.placePenguin(e.dst),this.postGameState()}catch(t){this.postIllegalPlacement(e.dst)}break;case"possibleMoves":this.postPossibleMoves(e.src);break;case"takeAction":this.takeAction(),this.postGameState()}}postIllegalMove(e,t){(0,this.postMessage)({type:"illegalMove",src:e,dst:t})}postIllegalPlacement(e){(0,this.postMessage)({type:"illegalPlacement",dst:e})}postPossibleMoves(e){(0,this.postMessage)({type:"possibleMoves",possibleMoves:this.getPossibleMoves(e)})}postGameState(){(0,this.postMessage)({type:"state",gameState:this.getState()})}postPlaceScores(e){const t=[];for(let e of this.game.draftable_cells()){const n=this.game.place_info(e);t.push({dst:e,visits:n.get_visits(),rewards:n.get_rewards()})}(0,this.postMessage)({type:"placeScores",activePlayer:e,placeScores:t})}postMoveScores(e){const t=[];for(let n of this.game.penguins(e))for(let e of this.game.possible_moves(n)){const r=this.game.move_info(n,e);t.push({src:n,dst:e,visits:r.get_visits(),rewards:r.get_rewards()})}(0,this.postMessage)({type:"moveScores",activePlayer:e,moveScores:t})}}const u=c;r()}catch(l){r(l)}}))},601:(e,t,n)=>{"use strict";n.d(t,{BS:()=>c,JQ:()=>i,Jh:()=>s,YP:()=>_,Zp:()=>r,_U:()=>o,c9:()=>a});const r=2,s=0,i=60,o=100,a=0,_=14e3,c=6e4},335:(e,t,n)=>{"use strict";n.a(e,(async(r,s)=>{try{var i,o=r([i=n(942)]),[i]=o.then?(await o)():o;await n.v(t,e.id,"a16cfdcab80ead46e8f7",{"./htmf_wasm_bg.js":{__wbindgen_string_new:i.h4,__wbg_new_693216e109162396:i.Ih,__wbg_stack_0ddaca5d1abfb52f:i.yq,__wbg_error_09919627ac0992f5:i.gk,__wbindgen_object_drop_ref:i.ug,__wbg_randomFillSync_59fcc2add91fe7b3:i._8,__wbg_getRandomValues_3e46aa268da0fed1:i.cR,__wbg_process_f2b73829dbd321da:i.MF,__wbindgen_is_object:i.Wl,__wbg_versions_cd82f79c98672a9f:i.Y8,__wbg_node_ee3f6da4130bd35f:i.GS,__wbindgen_is_string:i.eY,__wbg_modulerequire_0a83c0c31d12d2c7:i.GX,__wbg_crypto_9e3521ed42436d35:i.cF,__wbg_msCrypto_c429c3f8f7a70bb5:i.vG,__wbg_newnoargs_be86524d73f67598:i.wg,__wbg_call_888d259a5fefc347:i.BT,__wbg_self_c6fbdfc2918d5e58:i.JX,__wbg_window_baec038b5ab35c54:i.xd,__wbg_globalThis_3f735a5746d41fbd:i.ud,__wbg_global_1bc0b39582740e95:i.PT,__wbindgen_is_undefined:i.XP,__wbindgen_object_clone_ref:i.m_,__wbg_buffer_397eaa4d72ee94dd:i.jp,__wbg_new_a7ce447f15ff496f:i.y4,__wbg_set_969ad0a60e51d320:i.YQ,__wbg_length_1eb8fc608a0d4cdb:i.A7,__wbg_newwithlength_929232475839a482:i.b0,__wbg_subarray_8b658422a224f479:i.uf,__wbindgen_throw:i.Or,__wbindgen_rethrow:i.nD,__wbindgen_memory:i.oH}}),s()}catch(e){s(e)}}),1)}}]);