let o;const d=new Array(128).fill(void 0);d.push(void 0,null,!0,!1);function _(n){return d[n]}let m=d.length;function W(n){n<132||(d[n]=m,m=n)}function l(n){const e=_(n);return W(n),e}const O=typeof TextDecoder<"u"?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};typeof TextDecoder<"u"&&O.decode();let w=null;function h(){return(w===null||w.buffer!==o.memory.buffer)&&(w=new Uint8Array(o.memory.buffer)),w}function y(n,e){return n=n>>>0,O.decode(h().slice(n,n+e))}function a(n){m===d.length&&d.push(d.length+1);const e=m;return m=d[e],d[e]=n,e}function x(n){const e=o.initThreadPool(n);return l(e)}let p=null;function c(){return(p===null||p.buffer!==o.memory.buffer)&&(p=new Int32Array(o.memory.buffer)),p}function v(n,e){return n=n>>>0,h().subarray(n/1,n/1+e)}let S=0;const R=typeof TextEncoder<"u"?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},C=function(n,e){const t=R.encode(n);return e.set(t),{read:n.length,written:t.length}};function F(n,e,t){if(t===void 0){const f=R.encode(n),b=e(f.length,1)>>>0;return h().subarray(b,b+f.length).set(f),S=f.length,b}let r=n.length,s=e(r,1)>>>0;const i=h();let g=0;for(;g<r;g++){const f=n.charCodeAt(g);if(f>127)break;i[s+g]=f}if(g!==r){g!==0&&(n=n.slice(g)),s=t(s,r,r=g+n.length*3,1)>>>0;const f=h().subarray(s+g,s+r),b=C(n,f);g+=b.written}return S=g,s}function u(n,e){try{return n.apply(this,e)}catch(t){o.__wbindgen_exn_store(a(t))}}class T{static __wrap(e){e=e>>>0;const t=Object.create(T.prototype);return t.__wbg_ptr=e,t}__destroy_into_raw(){const e=this.__wbg_ptr;return this.__wbg_ptr=0,e}free(){const e=this.__destroy_into_raw();o.__wbg_game_free(e)}static new(){const e=o.game_new();return T.__wrap(e)}is_drafting(){return o.game_is_drafting(this.__wbg_ptr)!==0}finished_drafting(){return o.game_finished_drafting(this.__wbg_ptr)!==0}game_over(){return o.game_game_over(this.__wbg_ptr)!==0}active_player(){const e=o.game_active_player(this.__wbg_ptr);return e===16777215?void 0:e}score(e){return o.game_score(this.__wbg_ptr,e)>>>0}turn(){return o.game_turn(this.__wbg_ptr)>>>0}num_fish(e){return o.game_num_fish(this.__wbg_ptr,e)>>>0}penguins(e){try{const i=o.__wbindgen_add_to_stack_pointer(-16);o.game_penguins(i,this.__wbg_ptr,e);var t=c()[i/4+0],r=c()[i/4+1],s=v(t,r).slice();return o.__wbindgen_free(t,r*1,1),s}finally{o.__wbindgen_add_to_stack_pointer(16)}}claimed(e){try{const i=o.__wbindgen_add_to_stack_pointer(-16);o.game_claimed(i,this.__wbg_ptr,e);var t=c()[i/4+0],r=c()[i/4+1],s=v(t,r).slice();return o.__wbindgen_free(t,r*1,1),s}finally{o.__wbindgen_add_to_stack_pointer(16)}}draftable_cells(){try{const s=o.__wbindgen_add_to_stack_pointer(-16);o.game_draftable_cells(s,this.__wbg_ptr);var e=c()[s/4+0],t=c()[s/4+1],r=v(e,t).slice();return o.__wbindgen_free(e,t*1,1),r}finally{o.__wbindgen_add_to_stack_pointer(16)}}possible_moves(e){try{const i=o.__wbindgen_add_to_stack_pointer(-16);o.game_possible_moves(i,this.__wbg_ptr,e);var t=c()[i/4+0],r=c()[i/4+1],s=v(t,r).slice();return o.__wbindgen_free(t,r*1,1),s}finally{o.__wbindgen_add_to_stack_pointer(16)}}place_penguin(e){try{const s=o.__wbindgen_add_to_stack_pointer(-16);o.game_place_penguin(s,this.__wbg_ptr,e);var t=c()[s/4+0],r=c()[s/4+1];if(r)throw l(t)}finally{o.__wbindgen_add_to_stack_pointer(16)}}move_penguin(e,t){try{const i=o.__wbindgen_add_to_stack_pointer(-16);o.game_move_penguin(i,this.__wbg_ptr,e,t);var r=c()[i/4+0],s=c()[i/4+1];if(s)throw l(r)}finally{o.__wbindgen_add_to_stack_pointer(16)}}playout(){o.game_playout(this.__wbg_ptr)}playout_n_times(e){o.game_playout_n_times(this.__wbg_ptr,e)}get_total_playouts(){return o.game_get_total_playouts(this.__wbg_ptr)>>>0}get_visits(){return o.game_get_visits(this.__wbg_ptr)}place_info(e){const t=o.game_place_info(this.__wbg_ptr,e);return M.__wrap(t)}move_info(e,t){const r=o.game_move_info(this.__wbg_ptr,e,t);return M.__wrap(r)}take_action(){try{const r=o.__wbindgen_add_to_stack_pointer(-16);o.game_take_action(r,this.__wbg_ptr);var e=c()[r/4+0],t=c()[r/4+1];if(t)throw l(e)}finally{o.__wbindgen_add_to_stack_pointer(16)}}tree_size(){return o.game_tree_size(this.__wbg_ptr)>>>0}}class M{static __wrap(e){e=e>>>0;const t=Object.create(M.prototype);return t.__wbg_ptr=e,t}__destroy_into_raw(){const e=this.__wbg_ptr;return this.__wbg_ptr=0,e}free(){const e=this.__destroy_into_raw();o.__wbg_moveinfo_free(e)}get_visits(){return o.moveinfo_get_visits(this.__wbg_ptr)}get_rewards(){return o.moveinfo_get_rewards(this.__wbg_ptr)}}async function I(n,e){if(typeof Response=="function"&&n instanceof Response){if(typeof WebAssembly.instantiateStreaming=="function")try{return await WebAssembly.instantiateStreaming(n,e)}catch(r){if(n.headers.get("Content-Type")!="application/wasm")console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",r);else throw r}const t=await n.arrayBuffer();return await WebAssembly.instantiate(t,e)}else{const t=await WebAssembly.instantiate(n,e);return t instanceof WebAssembly.Instance?{instance:t,module:n}:t}}function N(){const n={};return n.wbg={},n.wbg.__wbindgen_object_drop_ref=function(e){l(e)},n.wbg.__wbindgen_string_new=function(e,t){const r=y(e,t);return a(r)},n.wbg.__wbg_new_abda76e883ba8a5f=function(){const e=new Error;return a(e)},n.wbg.__wbg_stack_658279fe44541cf6=function(e,t){const r=_(t).stack,s=F(r,o.__wbindgen_malloc,o.__wbindgen_realloc),i=S;c()[e/4+1]=i,c()[e/4+0]=s},n.wbg.__wbg_error_f851667af71bcfc6=function(e,t){let r,s;try{r=e,s=t,console.error(y(e,t))}finally{o.__wbindgen_free(r,s,1)}},n.wbg.__wbg_crypto_58f13aa23ffcb166=function(e){const t=_(e).crypto;return a(t)},n.wbg.__wbindgen_is_object=function(e){const t=_(e);return typeof t=="object"&&t!==null},n.wbg.__wbg_process_5b786e71d465a513=function(e){const t=_(e).process;return a(t)},n.wbg.__wbg_versions_c2ab80650590b6a2=function(e){const t=_(e).versions;return a(t)},n.wbg.__wbg_node_523d7bd03ef69fba=function(e){const t=_(e).node;return a(t)},n.wbg.__wbindgen_is_string=function(e){return typeof _(e)=="string"},n.wbg.__wbg_msCrypto_abcb1295e768d1f2=function(e){const t=_(e).msCrypto;return a(t)},n.wbg.__wbg_require_2784e593a4674877=function(){return u(function(){const e=module.require;return a(e)},arguments)},n.wbg.__wbindgen_is_function=function(e){return typeof _(e)=="function"},n.wbg.__wbg_getRandomValues_504510b5564925af=function(){return u(function(e,t){_(e).getRandomValues(_(t))},arguments)},n.wbg.__wbg_randomFillSync_a0d98aa11c81fe89=function(){return u(function(e,t){_(e).randomFillSync(l(t))},arguments)},n.wbg.__wbg_newnoargs_ccdcae30fd002262=function(e,t){const r=new Function(y(e,t));return a(r)},n.wbg.__wbg_call_669127b9d730c650=function(){return u(function(e,t){const r=_(e).call(_(t));return a(r)},arguments)},n.wbg.__wbindgen_object_clone_ref=function(e){const t=_(e);return a(t)},n.wbg.__wbg_self_3fad056edded10bd=function(){return u(function(){const e=self.self;return a(e)},arguments)},n.wbg.__wbg_window_a4f46c98a61d4089=function(){return u(function(){const e=window.window;return a(e)},arguments)},n.wbg.__wbg_globalThis_17eff828815f7d84=function(){return u(function(){const e=globalThis.globalThis;return a(e)},arguments)},n.wbg.__wbg_global_46f939f6541643c5=function(){return u(function(){const e=global.global;return a(e)},arguments)},n.wbg.__wbindgen_is_undefined=function(e){return _(e)===void 0},n.wbg.__wbg_call_53fc3abd42e24ec8=function(){return u(function(e,t,r){const s=_(e).call(_(t),_(r));return a(s)},arguments)},n.wbg.__wbg_resolve_a3252b2860f0a09e=function(e){const t=Promise.resolve(_(e));return a(t)},n.wbg.__wbg_buffer_344d9b41efe96da7=function(e){const t=_(e).buffer;return a(t)},n.wbg.__wbg_newwithbyteoffsetandlength_2dc04d99088b15e3=function(e,t,r){const s=new Uint8Array(_(e),t>>>0,r>>>0);return a(s)},n.wbg.__wbg_new_d8a000788389a31e=function(e){const t=new Uint8Array(_(e));return a(t)},n.wbg.__wbg_set_dcfd613a3420f908=function(e,t,r){_(e).set(_(t),r>>>0)},n.wbg.__wbg_newwithlength_13b5319ab422dcf6=function(e){const t=new Uint8Array(e>>>0);return a(t)},n.wbg.__wbg_subarray_6ca5cfa7fbb9abbe=function(e,t,r){const s=_(e).subarray(t>>>0,r>>>0);return a(s)},n.wbg.__wbindgen_throw=function(e,t){throw new Error(y(e,t))},n.wbg.__wbindgen_memory=function(){const e=o.memory;return a(e)},n}function j(n,e){n.wbg.memory=e||new WebAssembly.Memory({initial:18,maximum:16384,shared:!0})}function D(n,e){return o=n.exports,L.__wbindgen_wasm_module=e,p=null,w=null,o.__wbindgen_start(),o}async function L(n,e){if(o!==void 0)return o;typeof n>"u"&&(n=new URL(""+new URL("htmf_wasm_bg-32k3AWNe.wasm",import.meta.url).href,import.meta.url));const t=N();(typeof n=="string"||typeof Request=="function"&&n instanceof Request||typeof URL=="function"&&n instanceof URL)&&(n=fetch(n)),j(t,e);const{instance:r,module:s}=await I(await n,t);return D(r,s)}const Y=2,z=0,k=1,P=8,G=7,B=8,V=G*(P/2)+B*(P/2),A=200,E=14e3,U=6e4;function H(n){const e=[];for(let i=0;i<V;++i)e.push(n.num_fish(i));const t=[],r=[],s=[];for(let i=0;i<Y;++i)t.push(n.score(i)),r.push([...n.penguins(i)]),s.push([...n.claimed(i)]);return{activePlayer:n.active_player(),modeType:n.finished_drafting()?"playing":"drafting",scores:t,turn:n.turn(),board:{fish:e,penguins:r,claimed:s}}}function K(n,e){if(n.active_player()!==z)return[];if(n.finished_drafting()){if(e!==void 0)return[...n.possible_moves(e)];const t=n.active_player();return t===void 0?[]:[...n.penguins(t)]}else return[...n.draftable_cells()]}class X{wasmInternals;game=T.new();postMessage;ponderer;ponderStartTime;totalCompletedPonderTimeMs=0;constructor(e,t){this.wasmInternals=e,this.postMessage=t,this.ponder(),this.postGameState({})}free(){this.stopPondering(),this.game.free(),this.totalCompletedPonderTimeMs=0}ponder(){this.ponderer===void 0&&(this.ponderStartTime=performance.now(),this.ponderer=self.setInterval(()=>{const e=this.game.active_player();if(this.game.get_visits()>=U||e===k){this.stopPondering();return}this.game.playout_n_times(A),e!==void 0&&this.postThinkingProgress({activePlayer:e,playoutsNeeded:U})}))}stopPondering(){this.ponderer!==void 0&&(clearInterval(this.ponderer),this.ponderer=void 0),this.ponderStartTime!==void 0&&(this.totalCompletedPonderTimeMs+=performance.now()-this.ponderStartTime,this.ponderStartTime=void 0)}placePenguin(e){this.game.place_penguin(e),this.ponder()}movePenguin(e,t){this.game.move_penguin(e,t),this.ponder()}playout(){this.game.playout()}takeAction(){const e=this.game.turn()<2?2*E:E;let t=performance.now();for(;this.game.get_visits()<e;){this.game.playout_n_times(A),this.totalCompletedPonderTimeMs+=performance.now()-t,t=performance.now();const r=this.game.active_player();r!==void 0&&this.postThinkingProgress({activePlayer:r,playoutsNeeded:e})}this.totalCompletedPonderTimeMs+=performance.now()-t,t=performance.now(),this.game.take_action(),this.postGameState({}),this.ponder()}getState(){return H(this.game)}getPossibleMoves(e){return K(this.game,e)}onMessage(e){console.log(`received request ${e.type}`);let t,r=!1;switch(e.type){case"getGameState":break;case"movePenguin":try{e.src===void 0?this.placePenguin(e.dst):this.movePenguin(e.src,e.dst)}catch{r=!0}break;case"getPossibleMoves":t=e.src;break}for(this.postGameState({src:t,lastMoveWasIllegal:r});this.game.active_player()===k;)this.takeAction()}postGameState({src:e,lastMoveWasIllegal:t}){t=t===!0;const r=this.postMessage;r({type:"gameState",gameState:this.getState(),possibleMoves:this.getPossibleMoves(e),lastMoveWasIllegal:t})}postThinkingProgress({activePlayer:e,playoutsNeeded:t}){const r=this.postMessage;r({type:"thinkingProgress",completed:this.game.get_visits(),required:t,totalPlayouts:this.game.get_total_playouts(),totalTimeThinkingMs:this.getTotalTimeThinkingMs(),memoryUsage:this.wasmInternals.memory.buffer.byteLength,treeSize:this.game.tree_size(),playerMoveScores:{player:e,moveScores:this.getMoveScores(e)}})}getMoveScores(e){const t=[];if(this.game.is_drafting())for(const r of this.game.draftable_cells()){const s=this.game.place_info(r);t.push({dst:r,visits:s.get_visits(),rewards:s.get_rewards()})}else for(const r of this.game.penguins(e))for(const s of this.game.possible_moves(r)){const i=this.game.move_info(r,s);t.push({src:r,dst:s,visits:i.get_visits(),rewards:i.get_rewards()})}return t}getTotalTimeThinkingMs(){let e=this.totalCompletedPonderTimeMs;return this.ponderStartTime!==void 0&&(e+=performance.now()-this.ponderStartTime),e}}const Z=new WebAssembly.Memory({initial:18,maximum:16384,shared:!0}),$=await L(void 0,Z);await x(navigator.hardwareConcurrency);const q=new X($,postMessage);onmessage=n=>{q.onMessage(n.data)};
//# sourceMappingURL=bot.worker-imZvenz9.js.map
