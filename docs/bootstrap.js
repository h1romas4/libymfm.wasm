(()=>{var e,t,r,n,o,i,a,s,c,u,d,l,p,f,b,m,v,h,g={466:(e,t,r)=>{Promise.all([r.e(754),r.e(269)]).then(r.bind(r,269)).catch((function(e){return console.error("Error importing `index.js`:",e)}))}},_={};function y(e){var t=_[e];if(void 0!==t)return t.exports;var r=_[e]={id:e,loaded:!1,exports:{}};return g[e](r,r.exports,y),r.loaded=!0,r.exports}y.m=g,y.c=_,y.n=e=>{var t=e&&e.__esModule?()=>e.default:()=>e;return y.d(t,{a:t}),t},y.d=(e,t)=>{for(var r in t)y.o(t,r)&&!y.o(e,r)&&Object.defineProperty(e,r,{enumerable:!0,get:t[r]})},y.f={},y.e=e=>Promise.all(Object.keys(y.f).reduce(((t,r)=>(y.f[r](e,t),t)),[])),y.u=e=>e+".bootstrap.js",y.g=function(){if("object"==typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"==typeof window)return window}}(),y.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),y.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),e={},t="wasm-vgm-player:",y.l=(r,n,o,i)=>{if(e[r])e[r].push(n);else{var a,s;if(void 0!==o)for(var c=document.getElementsByTagName("script"),u=0;u<c.length;u++){var d=c[u];if(d.getAttribute("src")==r||d.getAttribute("data-webpack")==t+o){a=d;break}}a||(s=!0,(a=document.createElement("script")).charset="utf-8",a.timeout=120,y.nc&&a.setAttribute("nonce",y.nc),a.setAttribute("data-webpack",t+o),a.src=r),e[r]=[n];var l=(t,n)=>{a.onerror=a.onload=null,clearTimeout(p);var o=e[r];if(delete e[r],a.parentNode&&a.parentNode.removeChild(a),o&&o.forEach((e=>e(n))),t)return t(n)},p=setTimeout(l.bind(null,void 0,{type:"timeout",target:a}),12e4);a.onerror=l.bind(null,a.onerror),a.onload=l.bind(null,a.onload),s&&document.head.appendChild(a)}},y.r=e=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},(()=>{var e;y.g.importScripts&&(e=y.g.location+"");var t=y.g.document;if(!e&&t&&(t.currentScript&&(e=t.currentScript.src),!e)){var r=t.getElementsByTagName("script");r.length&&(e=r[r.length-1].src)}if(!e)throw new Error("Automatic publicPath is not supported in this browser");e=e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\/[^\/]+$/,"/"),y.p=e})(),(()=>{var e={179:0};y.f.j=(t,r)=>{var n=y.o(e,t)?e[t]:void 0;if(0!==n)if(n)r.push(n[2]);else{var o=new Promise(((r,o)=>n=e[t]=[r,o]));r.push(n[2]=o);var i=y.p+y.u(t),a=new Error;y.l(i,(r=>{if(y.o(e,t)&&(0!==(n=e[t])&&(e[t]=void 0),n)){var o=r&&("load"===r.type?"missing":r.type),i=r&&r.target&&r.target.src;a.message="Loading chunk "+t+" failed.\n("+o+": "+i+")",a.name="ChunkLoadError",a.type=o,a.request=i,n[1](a)}}),"chunk-"+t,t)}};var t=(t,r)=>{var n,o,[i,a,s]=r,c=0;if(i.some((t=>0!==e[t]))){for(n in a)y.o(a,n)&&(y.m[n]=a[n]);s&&s(y)}for(t&&t(r);c<i.length;c++)o=i[c],y.o(e,o)&&e[o]&&e[o][0](),e[i[c]]=0},r=self.webpackChunkwasm_vgm_player=self.webpackChunkwasm_vgm_player||[];r.forEach(t.bind(null,0)),r.push=t.bind(null,r.push.bind(r))})(),m={},v={944:function(){return{wasi_snapshot_preview1:{fd_seek:function(e,t,n,o){return void 0===r&&(r=y.c[913].exports),r.Cf(e,t,n,o)},fd_write:function(e,t,r,o){return void 0===n&&(n=y.c[913].exports),n.UX(e,t,r,o)},fd_close:function(e){return void 0===o&&(o=y.c[913].exports),o.yU(e)},environ_sizes_get:function(e,t){return void 0===d&&(d=y.c[913].exports),d.Vn(e,t)},proc_exit:function(e){return void 0===l&&(l=y.c[913].exports),l.lO(e)},environ_get:function(e,t){return void 0===p&&(p=y.c[913].exports),p.NM(e,t)},fd_write:function(e,t,r,n){return void 0===f&&(f=y.c[913].exports),f.UX(e,t,r,n)},random_get:function(e,t){return void 0===b&&(b=y.c[913].exports),b.H4(e,t)}},"./libymfm_bg.js":{__wbg_new_59cb74e423758ede:function(){return void 0===i&&(i=y.c[629].exports),i.h9()},__wbg_stack_558ba5917b466edd:function(e,t){return void 0===a&&(a=y.c[629].exports),a.Dz(e,t)},__wbg_error_4bb6c2a97407129a:function(e){return void 0===s&&(s=y.c[629].exports),s.kF(e)},__wbindgen_object_drop_ref:function(e){return void 0===c&&(c=y.c[629].exports),c.ug(e)},__wbindgen_throw:function(e,t){return void 0===u&&(u=y.c[629].exports),u.Or(e,t)}}}}},h={269:[944]},y.w={},y.f.wasm=function(e,t){(h[e]||[]).forEach((function(r,n){var o=m[r];if(o)t.push(o);else{var i,a=v[r](),s=fetch(y.p+""+{269:{944:"4074f966d9f344490d8a"}}[e][r]+".module.wasm");i=a&&"function"==typeof a.then&&"function"==typeof WebAssembly.compileStreaming?Promise.all([WebAssembly.compileStreaming(s),a]).then((function(e){return WebAssembly.instantiate(e[0],e[1])})):"function"==typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(s,a):s.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,a)})),t.push(m[r]=i.then((function(e){return y.w[r]=(e.instance||e).exports})))}}))},y(466)})();