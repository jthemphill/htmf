(()=>{var e,t,r,n,o,i,c,s,a,u,f,d,p,b,_,l,g,m={380:(e,t,r)=>{const n=self;r.e(331).then(r.bind(r,331)).catch((e=>console.error("Error importing `Bot.ts`:",e))).then((e=>{const t=new e.default(n.postMessage);n.onmessage=e=>{t.onMessage(e.data)},t.init()}))}},h={};function w(e){if(h[e])return h[e].exports;var t=h[e]={id:e,loaded:!1,exports:{}};return m[e](t,t.exports,w),t.loaded=!0,t.exports}w.m=m,w.c=h,w.d=(e,t)=>{for(var r in t)w.o(t,r)&&!w.o(e,r)&&Object.defineProperty(e,r,{enumerable:!0,get:t[r]})},w.f={},w.e=e=>Promise.all(Object.keys(w.f).reduce(((t,r)=>(w.f[r](e,t),t)),[])),w.u=e=>e+".index.js",w.g=function(){if("object"==typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"==typeof window)return window}}(),w.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),w.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),w.r=e=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},(()=>{var e;w.g.importScripts&&(e=w.g.location+"");var t=w.g.document;if(!e&&t&&(t.currentScript&&(e=t.currentScript.src),!e)){var r=t.getElementsByTagName("script");r.length&&(e=r[r.length-1].src)}if(!e)throw new Error("Automatic publicPath is not supported in this browser");e=e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\/[^\/]+$/,"/"),w.p=e})(),(()=>{var e={380:1};w.f.i=(t,r)=>{e[t]||importScripts(""+w.u(t))};var t=self.webpackChunkhtmf=self.webpackChunkhtmf||[],r=t.push.bind(t);t.push=t=>{var[n,o,i]=t;for(var c in o)w.o(o,c)&&(w.m[c]=o[c]);for(i&&i(w);n.length;)e[n.pop()]=1;r(t)}})(),_={},l={45:function(){return{"./htmf_wasm_bg.js":{__wbindgen_string_new:function(t,r){return void 0===e&&(e=w.c[942].exports),e.h4(t,r)},__wbg_new_59cb74e423758ede:function(){return void 0===t&&(t=w.c[942].exports),t.h9()},__wbg_stack_558ba5917b466edd:function(e,t){return void 0===r&&(r=w.c[942].exports),r.Dz(e,t)},__wbg_error_4bb6c2a97407129a:function(e,t){return void 0===n&&(n=w.c[942].exports),n.kF(e,t)},__wbindgen_object_drop_ref:function(e){return void 0===o&&(o=w.c[942].exports),o.ug(e)},__wbg_getRandomValues_f5e14ab7ac8e995d:function(e,t,r){return void 0===i&&(i=w.c[942].exports),i.f4(e,t,r)},__wbg_randomFillSync_d5bd2d655fdf256a:function(e,t,r){return void 0===c&&(c=w.c[942].exports),c.oS(e,t,r)},__wbg_self_1b7a39e3a92c949c:function(){return void 0===s&&(s=w.c[942].exports),s.I9()},__wbg_require_604837428532a733:function(e,t){return void 0===a&&(a=w.c[942].exports),a.ii(e,t)},__wbg_crypto_968f1772287e2df0:function(e){return void 0===u&&(u=w.c[942].exports),u.to(e)},__wbindgen_is_undefined:function(e){return void 0===f&&(f=w.c[942].exports),f.XP(e)},__wbg_getRandomValues_a3d34b4fee3c2869:function(e){return void 0===d&&(d=w.c[942].exports),d.py(e)},__wbindgen_throw:function(e,t){return void 0===p&&(p=w.c[942].exports),p.Or(e,t)},__wbindgen_rethrow:function(e){return void 0===b&&(b=w.c[942].exports),b.nD(e)}}}}},g={331:[45]},w.w={},w.f.wasm=function(e,t){(g[e]||[]).forEach((function(r,n){var o=_[r];if(o)t.push(o);else{var i,c=l[r](),s=fetch(w.p+""+{331:{45:"3e1a2c4ca01e31280e0f"}}[e][r]+".module.wasm");i=c instanceof Promise&&"function"==typeof WebAssembly.compileStreaming?Promise.all([WebAssembly.compileStreaming(s),c]).then((function(e){return WebAssembly.instantiate(e[0],e[1])})):"function"==typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(s,c):s.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,c)})),t.push(_[r]=i.then((function(e){return w.w[r]=(e.instance||e).exports})))}}))},w(380)})();