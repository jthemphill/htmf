(()=>{"use strict";var e,t,r,n,o,i,c,s,u,_,a,f,d,b,p,l,g,w,m,h,v,x,y,S,j,O={708:(e,t,r)=>{e.exports=(async()=>{const e=(await r.e(453).then(r.bind(r,453))).default,t=self,n=new e(t.postMessage);t.onmessage=e=>{n.onMessage(e.data)},n.init()})()}},P={};function E(e){if(P[e])return P[e].exports;var t=P[e]={id:e,loaded:!1,exports:{}};return O[e](t,t.exports,E),t.loaded=!0,t.exports}E.m=O,E.c=P,E.d=(e,t)=>{for(var r in t)E.o(t,r)&&!E.o(e,r)&&Object.defineProperty(e,r,{enumerable:!0,get:t[r]})},E.f={},E.e=e=>Promise.all(Object.keys(E.f).reduce(((t,r)=>(E.f[r](e,t),t)),[])),E.u=e=>e+".index.js",E.g=function(){if("object"==typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"==typeof window)return window}}(),E.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),E.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),E.r=e=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},(()=>{var e;E.g.importScripts&&(e=E.g.location+"");var t=E.g.document;if(!e&&t&&(t.currentScript&&(e=t.currentScript.src),!e)){var r=t.getElementsByTagName("script");r.length&&(e=r[r.length-1].src)}if(!e)throw new Error("Automatic publicPath is not supported in this browser");e=e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\/[^\/]+$/,"/"),E.p=e})(),(()=>{var e={708:1};E.f.i=(t,r)=>{e[t]||importScripts(""+E.u(t))};var t=globalThis.webpackChunkhtmf=globalThis.webpackChunkhtmf||[],r=t.push.bind(t);t.push=t=>{var[n,o,i]=t;for(var c in o)E.o(o,c)&&(E.m[c]=o[c]);for(i&&i(E);n.length;)e[n.pop()]=1;r(t)}})(),y={},S={45:function(){return{"./htmf_wasm_bg.js":{__wbindgen_string_new:function(t,r){return void 0===e&&(e=E.c[942].exports),e.h4(t,r)},__wbg_new_59cb74e423758ede:function(){return void 0===t&&(t=E.c[942].exports),t.h9()},__wbg_stack_558ba5917b466edd:function(e,t){return void 0===r&&(r=E.c[942].exports),r.Dz(e,t)},__wbg_error_4bb6c2a97407129a:function(e,t){return void 0===n&&(n=E.c[942].exports),n.kF(e,t)},__wbindgen_object_drop_ref:function(e){return void 0===o&&(o=E.c[942].exports),o.ug(e)},__wbg_getRandomValues_57e4008f45f0e105:function(e,t){return void 0===i&&(i=E.c[942].exports),i.E_(e,t)},__wbg_randomFillSync_d90848a552cbd666:function(e,t,r){return void 0===c&&(c=E.c[942].exports),c.L7(e,t,r)},__wbg_self_f865985e662246aa:function(){return void 0===s&&(s=E.c[942].exports),s.MY()},__wbg_static_accessor_MODULE_39947eb3fe77895f:function(){return void 0===u&&(u=E.c[942].exports),u.f$()},__wbg_require_c59851dfa0dc7e78:function(e,t,r){return void 0===_&&(_=E.c[942].exports),_.sV(e,t,r)},__wbg_crypto_bfb05100db79193b:function(e){return void 0===a&&(a=E.c[942].exports),a._y(e)},__wbg_msCrypto_f6dddc6ae048b7e2:function(e){return void 0===f&&(f=E.c[942].exports),f.Yj(e)},__wbindgen_is_undefined:function(e){return void 0===d&&(d=E.c[942].exports),d.XP(e)},__wbg_buffer_bc64154385c04ac4:function(e){return void 0===b&&(b=E.c[942].exports),b.gW(e)},__wbg_length_e9f6f145de2fede5:function(e){return void 0===p&&(p=E.c[942].exports),p.FW(e)},__wbg_new_22a33711cf65b661:function(e){return void 0===l&&(l=E.c[942].exports),l.xC(e)},__wbg_set_b29de3f25280c6ec:function(e,t,r){return void 0===g&&(g=E.c[942].exports),g.GD(e,t,r)},__wbg_newwithlength_48451d71403bfede:function(e){return void 0===w&&(w=E.c[942].exports),w.G1(e)},__wbg_subarray_6b2dd31c84ee881f:function(e,t,r){return void 0===m&&(m=E.c[942].exports),m.X(e,t,r)},__wbindgen_throw:function(e,t){return void 0===h&&(h=E.c[942].exports),h.Or(e,t)},__wbindgen_rethrow:function(e){return void 0===v&&(v=E.c[942].exports),v.nD(e)},__wbindgen_memory:function(){return void 0===x&&(x=E.c[942].exports),x.oH()}}}}},j={453:[45]},E.w={},E.f.wasm=function(e,t){(j[e]||[]).forEach((function(r,n){var o=y[r];if(o)t.push(o);else{var i,c=S[r](),s=fetch(E.p+""+{453:{45:"419e7af0ad4c096411ed"}}[e][r]+".module.wasm");i=c instanceof Promise&&"function"==typeof WebAssembly.compileStreaming?Promise.all([WebAssembly.compileStreaming(s),c]).then((function(e){return WebAssembly.instantiate(e[0],e[1])})):"function"==typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(s,c):s.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,c)})),t.push(y[r]=i.then((function(e){return E.w[r]=(e.instance||e).exports})))}}))},E(708)})();