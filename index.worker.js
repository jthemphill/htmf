!function(e){self.webpackChunk=function(n,r){for(var o in r)e[o]=r[o];for(;n.length;)t[n.pop()]=1};var n={},t={0:1},r={};var o={3:function(){return{"./htmf_wasm_bg.js":{__wbindgen_string_new:function(e,t){return n[2].exports.n(e,t)},__wbg_new_59cb74e423758ede:function(){return n[2].exports.f()},__wbg_stack_558ba5917b466edd:function(e,t){return n[2].exports.j(e,t)},__wbg_error_4bb6c2a97407129a:function(e,t){return n[2].exports.c(e,t)},__wbindgen_object_drop_ref:function(e){return n[2].exports.l(e)},__wbg_self_1b7a39e3a92c949c:function(){return n[2].exports.i()},__wbg_require_604837428532a733:function(e,t){return n[2].exports.h(e,t)},__wbg_crypto_968f1772287e2df0:function(e){return n[2].exports.b(e)},__wbindgen_is_undefined:function(e){return n[2].exports.k(e)},__wbg_getRandomValues_a3d34b4fee3c2869:function(e){return n[2].exports.d(e)},__wbg_getRandomValues_f5e14ab7ac8e995d:function(e,t,r){return n[2].exports.e(e,t,r)},__wbg_randomFillSync_d5bd2d655fdf256a:function(e,t,r){return n[2].exports.g(e,t,r)},__wbindgen_throw:function(e,t){return n[2].exports.o(e,t)},__wbindgen_rethrow:function(e){return n[2].exports.m(e)}}}}};function i(t){if(n[t])return n[t].exports;var r=n[t]={i:t,l:!1,exports:{}};return e[t].call(r.exports,r,r.exports,i),r.l=!0,r.exports}i.e=function(e){var n=[];return n.push(Promise.resolve().then((function(){t[e]||importScripts(i.p+""+e+".index.worker.js")}))),({1:[3]}[e]||[]).forEach((function(e){var t=r[e];if(t)n.push(t);else{var u,s=o[e](),f=fetch(i.p+""+{3:"679430a049b5076475e5"}[e]+".module.wasm");if(s instanceof Promise&&"function"==typeof WebAssembly.compileStreaming)u=Promise.all([WebAssembly.compileStreaming(f),s]).then((function(e){return WebAssembly.instantiate(e[0],e[1])}));else if("function"==typeof WebAssembly.instantiateStreaming)u=WebAssembly.instantiateStreaming(f,s);else{u=f.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,s)}))}n.push(r[e]=u.then((function(n){return i.w[e]=(n.instance||n).exports})))}})),Promise.all(n)},i.m=e,i.c=n,i.d=function(e,n,t){i.o(e,n)||Object.defineProperty(e,n,{enumerable:!0,get:t})},i.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},i.t=function(e,n){if(1&n&&(e=i(e)),8&n)return e;if(4&n&&"object"==typeof e&&e&&e.__esModule)return e;var t=Object.create(null);if(i.r(t),Object.defineProperty(t,"default",{enumerable:!0,value:e}),2&n&&"string"!=typeof e)for(var r in e)i.d(t,r,function(n){return e[n]}.bind(null,r));return t},i.n=function(e){var n=e&&e.__esModule?function(){return e.default}:function(){return e};return i.d(n,"a",n),n},i.o=function(e,n){return Object.prototype.hasOwnProperty.call(e,n)},i.p="",i.w={},i(i.s=0)}([function(e,n,t){const r=self;t.e(1).then(t.t.bind(null,1,7)).catch(e=>console.error("Error importing `Bot.ts`:",e)).then(e=>{const n=new e.default(r.postMessage);r.onmessage=e=>{n.onMessage(e.data)},n.init()})}]);