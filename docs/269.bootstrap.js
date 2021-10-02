"use strict";(self.webpackChunkwasm_vgm_player=self.webpackChunkwasm_vgm_player||[]).push([[269],{269:(e,t,n)=>{n.r(t);var r,a,o,i,l,c,u,f,_=n(944),s=n(629),d=2048,g=768,m=576,y="#00a040",h="#e60012",p="16px sans-serif",w=null,v=0,k=[],b=44100,x=null,A=null,T=null;if((u=document.getElementById("screen")).setAttribute("width",g),u.setAttribute("height",m),(window.devicePixelRatio?window.devicePixelRatio:1)>1&&window.screen.width<g&&(u.style.width="320px",u.style.heigth="240px"),f=u.getContext("2d"),""!=location.hash){var E=location.hash.match(/^#s=(\d+)/);null!=E&&((b=parseInt(E[1]))!=b||44100!=b&&48e3!=b&&88200!=b&&96e3!=b)&&(b=44100)}var j=2*b/d;fetch("./vgm/ym2612.vgm").then((function(e){return e.arrayBuffer()})).then((function(e){L(e)})).then((function(){u.addEventListener("click",O,!1),u.addEventListener("dragover",(function(e){return D(e),u.style.border="4px dotted #333333",!1}),!1),u.addEventListener("dragleave",(function(e){return D(e),u.style.border="none",!1})),u.addEventListener("drop",P,!1),r=1,a=q({track_name:"WebAssembly 👾 VGM Player",track_name_j:"",game_name:"",game_name_j:"YM2612 sample VGM",track_author:"@h1romas4",track_author_j:""}),x=new(window.AudioContext||window.webkitAudioContext)({sampleRate:b}),x=null,C()}));var M=function(e,t){var n=(g-f.measureText(e).width)/2;f.fillText(e,n,t)},C=function(){f.fillStyle="rgb(0, 0, 0)",f.fillRect(0,0,g,m),f.font="bold 28px sans-serif",f.fillStyle=h,M("WebAssembly 🎮 VGM Player",160),f.fillStyle=y,f.font="15px sans-serif",M("YM2151 | YM2203 | YM2149 | YM2413 | YM2612 | SN76489(MD) | PWM(32x) | SEGAPCM",224),f.font="20px sans-serif",M("🎵 DRAG AND DROP VGM(vgm/vgz) HEAR",256),M("OR CLICK(TAP) TO PLAY SAMPLE VGM",320),V()},D=function(e){e.preventDefault(),e.stopPropagation()},P=function e(t){D(t),u.removeEventListener("drop",e,!1),u.style.border="none";var n={},a=t.dataTransfer.files;return[].forEach.call(a,(function(t){var o=new FileReader;o.onload=function(){n[t.name]=o.result,Object.keys(n).length>=a.length&&(u.addEventListener("drop",e,!1),k=[],Object.keys(n).sort().forEach((function(e){k.push(n[e])})),r=k.length,S())},o.readAsArrayBuffer(t)})),!1},S=function e(){k.length<=0||(L(k.shift())?O():e())},q=function(e){return e.game_track_name=[e.game_name,e.track_name].filter((function(e){return""!=e})).join(" | "),e.game_track_name_j=[e.game_name_j,e.track_name_j].filter((function(e){return""!=e})).join(" / "),e.track_author_full=[e.track_author,e.track_author_j].filter((function(e){return""!=e})).join(" - "),f.font=p,e.game_track_name_left=(g-f.measureText(e.game_track_name).width)/2,e.game_track_name_j_left=(g-f.measureText(e.game_track_name_j).width)/2,e.track_author_full_left=(g-f.measureText(e.track_author_full).width)/2,e},L=function(e){return null!=w&&w.free(),w=new s.n9(b,d,e.byteLength),new Uint8Array(_.memory.buffer,w.get_seq_data_ref(),e.byteLength).set(new Uint8Array(e)),w.init()?(a=q(JSON.parse(w.get_seq_gd3())),!0):(w.free(),!1)},R=function(){null!=i&&i.disconnect(),null!=o&&o.disconnect(),null!=A&&A.disconnect(),i=null,A=null,o=null},O=function e(){u.removeEventListener("click",e,!1),R(),null==x&&(x=new(window.AudioContext||window.webkitAudioContext)({sampleRate:b})),A=x.createScriptProcessor(d,2,2),v=0;var t=!1;A.onaudioprocess=function(e){if(t)return R(),void S();var n;try{n=w.play()}catch(e){alert("ymfm:\n\nAn unexpected error has occurred. System has stoped. Please reload brwoser.\n\n".concat(e)),t=!0}e.outputBuffer.getChannelData(0).set(new Float32Array(_.memory.buffer,w.get_sampling_l_ref(),d)),e.outputBuffer.getChannelData(1).set(new Float32Array(_.memory.buffer,w.get_sampling_r_ref(),d)),n>=2&&(0==v&&n>2?t=!0:(0==v&&(o.gain.setValueAtTime(1,x.currentTime),o.gain.linearRampToValueAtTime(0,x.currentTime+2)),++v>j&&(t=!0)))},o=x.createGain(),A.connect(o),o.connect(x.destination),o.gain.setValueAtTime(1,x.currentTime),i=x.createAnalyser(),c=i.frequencyBinCount,l=new Uint8Array(c),i.getByteTimeDomainData(l),o.connect(i),null!=T&&(window.cancelAnimationFrame(T),T=null),B()},B=function e(){if(T=window.requestAnimationFrame(e),f.fillStyle="rgb(0, 0, 0)",f.fillRect(0,0,g,m),null!=i){i.getByteFrequencyData(l),f.lineWidth=1,f.beginPath(),f.strokeStyle=h;var t=Math.round(c/192);f.setLineDash([2,1]),f.lineWidth=4;for(var n=0;n<c;n+=t)f.beginPath(),f.moveTo(n+2,m),f.lineTo(n+2,m-1.5*l[n]),f.stroke();f.stroke()}f.font="12px monospace",f.fillStyle=y,r>=1&&M("Track "+(r-k.length)+" / "+r,192),f.font=p,f.fillText(a.game_track_name,a.game_track_name_left,224),f.fillText(a.game_track_name_j,a.game_track_name_j_left,256),f.fillText(a.track_author_full,a.track_author_full_left,288),V()},V=function(){if(44100!=b){var e=" HD:"+b+" ";f.font="16px sans-serif";var t=f.measureText(e);f.fillStyle=y,f.fillRect(g-t.width,0,g,18),f.fillStyle="rgb(0, 0, 0)",f.fillText(e,g-t.width,16)}}},913:(e,t,n)=>{n.d(t,{Cf:()=>r,UX:()=>a,yU:()=>o,Vn:()=>i,lO:()=>l,NM:()=>c,H4:()=>u});var r=function(e,t,n,r){console.log("fd_seek: ".concat(e,", ").concat(t,", ").concat(n,", ").concat(r))},a=function(e,t,n,r){console.log("fd_write: ".concat(e,", ").concat(t,", ").concat(n,", ").concat(r))},o=function(e){console.log("fd_close: ".concat(e))},i=function(e,t){console.log("environ_sizes_get: ".concat(e,", ").concat(t))},l=function(e){console.log("proc_exit: ".concat(e))},c=function(e,t){console.log("environ_get: ".concat(e,", ").concat(t))},u=function(e,t){return console.log("random_get: ".concat(e,", ").concat(t)),0}},629:(e,t,n)=>{n.d(t,{n9:()=>h,h9:()=>p,Dz:()=>w,kF:()=>v,ug:()=>k,Or:()=>b});var r=n(944);function a(e,t){for(var n=0;n<t.length;n++){var r=t[n];r.enumerable=r.enumerable||!1,r.configurable=!0,"value"in r&&(r.writable=!0),Object.defineProperty(e,r.key,r)}}e=n.hmd(e);var o=new Array(32).fill(void 0);function i(e){return o[e]}o.push(void 0,null,!0,!1);var l=o.length;var c=new("undefined"==typeof TextDecoder?(0,e.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});c.decode();var u=null;function f(){return null!==u&&u.buffer===r.memory.buffer||(u=new Uint8Array(r.memory.buffer)),u}function _(e,t){return c.decode(f().subarray(e,e+t))}var s=null;function d(){return null!==s&&s.buffer===r.memory.buffer||(s=new Int32Array(r.memory.buffer)),s}var g=0,m=new("undefined"==typeof TextEncoder?(0,e.require)("util").TextEncoder:TextEncoder)("utf-8"),y="function"==typeof m.encodeInto?function(e,t){return m.encodeInto(e,t)}:function(e,t){var n=m.encode(e);return t.set(n),{read:e.length,written:n.length}},h=function(){function e(t,n,a){!function(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}(this,e);var o=r.wgmplay_from(t,n,a);return e.__wrap(o)}var t,n,o;return t=e,o=[{key:"__wrap",value:function(t){var n=Object.create(e.prototype);return n.ptr=t,n}}],(n=[{key:"__destroy_into_raw",value:function(){var e=this.ptr;return this.ptr=0,e}},{key:"free",value:function(){var e=this.__destroy_into_raw();r.__wbg_wgmplay_free(e)}},{key:"get_seq_data_ref",value:function(){return r.wgmplay_get_seq_data_ref(this.ptr)}},{key:"get_sampling_l_ref",value:function(){return r.wgmplay_get_sampling_l_ref(this.ptr)}},{key:"get_sampling_r_ref",value:function(){return r.wgmplay_get_sampling_r_ref(this.ptr)}},{key:"get_seq_header",value:function(){try{var e=r.__wbindgen_add_to_stack_pointer(-16);r.wgmplay_get_seq_header(e,this.ptr);var t=d()[e/4+0],n=d()[e/4+1];return _(t,n)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(t,n)}}},{key:"get_seq_gd3",value:function(){try{var e=r.__wbindgen_add_to_stack_pointer(-16);r.wgmplay_get_seq_gd3(e,this.ptr);var t=d()[e/4+0],n=d()[e/4+1];return _(t,n)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(t,n)}}},{key:"init",value:function(){return 0!==r.wgmplay_init(this.ptr)}},{key:"play",value:function(){return r.wgmplay_play(this.ptr)>>>0}}])&&a(t.prototype,n),o&&a(t,o),e}();function p(){return function(e){l===o.length&&o.push(o.length+1);var t=l;return l=o[t],o[t]=e,t}(new Error)}function w(e,t){var n=function(e,t,n){if(void 0===n){var r=m.encode(e),a=t(r.length);return f().subarray(a,a+r.length).set(r),g=r.length,a}for(var o=e.length,i=t(o),l=f(),c=0;c<o;c++){var u=e.charCodeAt(c);if(u>127)break;l[i+c]=u}if(c!==o){0!==c&&(e=e.slice(c)),i=n(i,o,o=c+3*e.length);var _=f().subarray(i+c,i+o);c+=y(e,_).written}return g=c,i}(i(t).stack,r.__wbindgen_malloc,r.__wbindgen_realloc),a=g;d()[e/4+1]=a,d()[e/4+0]=n}function v(e,t){try{console.error(_(e,t))}finally{r.__wbindgen_free(e,t)}}function k(e){var t;i(t=e),function(e){e<36||(o[e]=l,l=e)}(t)}function b(e,t){throw new Error(_(e,t))}},944:(e,t,n)=>{var r=n.w[e.id];e.exports=r,n(913),n(629),r[""]()}}]);