"use strict";(self.webpackChunkwasm_vgm_player=self.webpackChunkwasm_vgm_player||[]).push([[269],{269:(e,t,n)=>{n.r(t);var r,a,o,i,l,c,u,f,_=n(944),s=n(629),d=768,g=576,m="#00a040",y="#e60012",h="16px sans-serif",p=null,w=0,v=[],k=44100,b=null,x=null,A=null;if((u=document.getElementById("screen")).setAttribute("width",d),u.setAttribute("height",g),(window.devicePixelRatio?window.devicePixelRatio:1)>1&&window.screen.width<d&&(u.style.width="320px",u.style.heigth="240px"),f=u.getContext("2d"),""!=location.hash){var T=location.hash.match(/^#s=(\d+)/);null!=T&&((k=parseInt(T[1]))!=k||44100!=k&&48e3!=k&&88200!=k&&96e3!=k)&&(k=44100)}var E=k>=88200?512:256,j=2*k/E;fetch("./vgm/ym2612.vgm").then((function(e){return e.arrayBuffer()})).then((function(e){L(e)})).then((function(){u.addEventListener("click",O,!1),u.addEventListener("dragover",(function(e){return D(e),u.style.border="4px dotted #333333",!1}),!1),u.addEventListener("dragleave",(function(e){return D(e),u.style.border="none",!1})),u.addEventListener("drop",P,!1),r=1,a=q({track_name:"WebAssembly 👾 VGM Player",track_name_j:"",game_name:"",game_name_j:"YM2612 sample VGM",track_author:"@h1romas4",track_author_j:""}),b=new(window.AudioContext||window.webkitAudioContext)({sampleRate:k}),b=null,C()}));var M=function(e,t){var n=(d-f.measureText(e).width)/2;f.fillText(e,n,t)},C=function(){f.fillStyle="rgb(0, 0, 0)",f.fillRect(0,0,d,g),f.font="bold 28px sans-serif",f.fillStyle=y,M("WebAssembly 🎮 VGM Player",160),f.fillStyle=m,f.font="15px sans-serif",M("YM2151 | YM2203 | YM2149 | YM2413 | YM2612 | SN76489(MD) | PWM(32x) | SEGAPCM",224),f.font="20px sans-serif",M("🎵 DRAG AND DROP VGM(vgm/vgz) HEAR",256),M("OR CLICK(TAP) TO PLAY SAMPLE VGM",320),V()},D=function(e){e.preventDefault(),e.stopPropagation()},P=function e(t){D(t),u.removeEventListener("drop",e,!1),u.style.border="none";var n={},a=t.dataTransfer.files;return[].forEach.call(a,(function(t){var o=new FileReader;o.onload=function(){n[t.name]=o.result,Object.keys(n).length>=a.length&&(u.addEventListener("drop",e,!1),v=[],Object.keys(n).sort().forEach((function(e){v.push(n[e])})),r=v.length,S())},o.readAsArrayBuffer(t)})),!1},S=function e(){v.length<=0||(L(v.shift())?O():e())},q=function(e){return e.game_track_name=[e.game_name,e.track_name].filter((function(e){return""!=e})).join(" | "),e.game_track_name_j=[e.game_name_j,e.track_name_j].filter((function(e){return""!=e})).join(" / "),e.track_author_full=[e.track_author,e.track_author_j].filter((function(e){return""!=e})).join(" - "),f.font=h,e.game_track_name_left=(d-f.measureText(e.game_track_name).width)/2,e.game_track_name_j_left=(d-f.measureText(e.game_track_name_j).width)/2,e.track_author_full_left=(d-f.measureText(e.track_author_full).width)/2,e},L=function(e){return null!=p&&p.free(),p=new s.n9(k,E,e.byteLength),new Uint8Array(_.memory.buffer,p.get_seq_data_ref(),e.byteLength).set(new Uint8Array(e)),p.init()?(a=q(JSON.parse(p.get_seq_gd3())),!0):(p.free(),!1)},R=function(){null!=i&&i.disconnect(),null!=o&&o.disconnect(),null!=x&&x.disconnect(),i=null,x=null,o=null},O=function e(){u.removeEventListener("click",e,!1),R(),null==b&&(b=new(window.AudioContext||window.webkitAudioContext)({sampleRate:k})),x=b.createScriptProcessor(E,2,2),w=0;var t=!1;x.onaudioprocess=function(e){if(t)return R(),void S();var n;try{n=p.play()}catch(e){alert("ymfm:\n\nAn unexpected error has occurred. System has stoped. Please reload brwoser.\n\n".concat(e)),t=!0}e.outputBuffer.getChannelData(0).set(new Float32Array(_.memory.buffer,p.get_sampling_l_ref(),E)),e.outputBuffer.getChannelData(1).set(new Float32Array(_.memory.buffer,p.get_sampling_r_ref(),E)),n>=2&&(0==w&&n>2?t=!0:(0==w&&(o.gain.setValueAtTime(1,b.currentTime),o.gain.linearRampToValueAtTime(0,b.currentTime+2)),++w>j&&(t=!0)))},o=b.createGain(),x.connect(o),o.connect(b.destination),o.gain.setValueAtTime(1,b.currentTime),i=b.createAnalyser(),c=i.frequencyBinCount,l=new Uint8Array(c),i.getByteTimeDomainData(l),o.connect(i),null!=A&&(window.cancelAnimationFrame(A),A=null),B()},B=function e(){if(A=window.requestAnimationFrame(e),f.fillStyle="rgb(0, 0, 0)",f.fillRect(0,0,d,g),null!=i){i.getByteFrequencyData(l),f.lineWidth=1,f.beginPath(),f.strokeStyle=y;var t=Math.round(c/192);f.setLineDash([2,1]),f.lineWidth=4;for(var n=0;n<c;n+=t)f.beginPath(),f.moveTo(n+2,g),f.lineTo(n+2,g-1.5*l[n]),f.stroke();f.stroke()}f.font="12px monospace",f.fillStyle=m,r>=1&&M("Track "+(r-v.length)+" / "+r,192),f.font=h,f.fillText(a.game_track_name,a.game_track_name_left,224),f.fillText(a.game_track_name_j,a.game_track_name_j_left,256),f.fillText(a.track_author_full,a.track_author_full_left,288),V()},V=function(){if(44100!=k){var e=" HD:"+k+" ";f.font="16px sans-serif";var t=f.measureText(e);f.fillStyle=m,f.fillRect(d-t.width,0,d,18),f.fillStyle="rgb(0, 0, 0)",f.fillText(e,d-t.width,16)}}},913:(e,t,n)=>{n.d(t,{Cf:()=>r,UX:()=>a,yU:()=>o,Vn:()=>i,lO:()=>l,NM:()=>c,H4:()=>u});var r=function(e,t,n,r){console.log("fd_seek: ".concat(e,", ").concat(t,", ").concat(n,", ").concat(r))},a=function(e,t,n,r){console.log("fd_write: ".concat(e,", ").concat(t,", ").concat(n,", ").concat(r))},o=function(e){console.log("fd_close: ".concat(e))},i=function(e,t){console.log("environ_sizes_get: ".concat(e,", ").concat(t))},l=function(e){console.log("proc_exit: ".concat(e))},c=function(e,t){console.log("environ_get: ".concat(e,", ").concat(t))},u=function(e,t){return console.log("random_get: ".concat(e,", ").concat(t)),0}},629:(e,t,n)=>{n.d(t,{n9:()=>h,h9:()=>p,Dz:()=>w,kF:()=>v,ug:()=>k,Or:()=>b});var r=n(944);function a(e,t){for(var n=0;n<t.length;n++){var r=t[n];r.enumerable=r.enumerable||!1,r.configurable=!0,"value"in r&&(r.writable=!0),Object.defineProperty(e,r.key,r)}}e=n.hmd(e);var o=new Array(32).fill(void 0);function i(e){return o[e]}o.push(void 0,null,!0,!1);var l=o.length;var c=new("undefined"==typeof TextDecoder?(0,e.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});c.decode();var u=null;function f(){return null!==u&&u.buffer===r.memory.buffer||(u=new Uint8Array(r.memory.buffer)),u}function _(e,t){return c.decode(f().subarray(e,e+t))}var s=null;function d(){return null!==s&&s.buffer===r.memory.buffer||(s=new Int32Array(r.memory.buffer)),s}var g=0,m=new("undefined"==typeof TextEncoder?(0,e.require)("util").TextEncoder:TextEncoder)("utf-8"),y="function"==typeof m.encodeInto?function(e,t){return m.encodeInto(e,t)}:function(e,t){var n=m.encode(e);return t.set(n),{read:e.length,written:n.length}},h=function(){function e(t,n,a){!function(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}(this,e);var o=r.wgmplay_from(t,n,a);return e.__wrap(o)}var t,n,o;return t=e,o=[{key:"__wrap",value:function(t){var n=Object.create(e.prototype);return n.ptr=t,n}}],(n=[{key:"__destroy_into_raw",value:function(){var e=this.ptr;return this.ptr=0,e}},{key:"free",value:function(){var e=this.__destroy_into_raw();r.__wbg_wgmplay_free(e)}},{key:"get_seq_data_ref",value:function(){return r.wgmplay_get_seq_data_ref(this.ptr)}},{key:"get_sampling_l_ref",value:function(){return r.wgmplay_get_sampling_l_ref(this.ptr)}},{key:"get_sampling_r_ref",value:function(){return r.wgmplay_get_sampling_r_ref(this.ptr)}},{key:"get_seq_header",value:function(){try{var e=r.__wbindgen_add_to_stack_pointer(-16);r.wgmplay_get_seq_header(e,this.ptr);var t=d()[e/4+0],n=d()[e/4+1];return _(t,n)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(t,n)}}},{key:"get_seq_gd3",value:function(){try{var e=r.__wbindgen_add_to_stack_pointer(-16);r.wgmplay_get_seq_gd3(e,this.ptr);var t=d()[e/4+0],n=d()[e/4+1];return _(t,n)}finally{r.__wbindgen_add_to_stack_pointer(16),r.__wbindgen_free(t,n)}}},{key:"init",value:function(){return 0!==r.wgmplay_init(this.ptr)}},{key:"play",value:function(){return r.wgmplay_play(this.ptr)>>>0}}])&&a(t.prototype,n),o&&a(t,o),e}();function p(){return function(e){l===o.length&&o.push(o.length+1);var t=l;return l=o[t],o[t]=e,t}(new Error)}function w(e,t){var n=function(e,t,n){if(void 0===n){var r=m.encode(e),a=t(r.length);return f().subarray(a,a+r.length).set(r),g=r.length,a}for(var o=e.length,i=t(o),l=f(),c=0;c<o;c++){var u=e.charCodeAt(c);if(u>127)break;l[i+c]=u}if(c!==o){0!==c&&(e=e.slice(c)),i=n(i,o,o=c+3*e.length);var _=f().subarray(i+c,i+o);c+=y(e,_).written}return g=c,i}(i(t).stack,r.__wbindgen_malloc,r.__wbindgen_realloc),a=g;d()[e/4+1]=a,d()[e/4+0]=n}function v(e,t){try{console.error(_(e,t))}finally{r.__wbindgen_free(e,t)}}function k(e){var t;i(t=e),function(e){e<36||(o[e]=l,l=e)}(t)}function b(e,t){throw new Error(_(e,t))}},944:(e,t,n)=>{var r=n.w[e.id];e.exports=r,n(913),n(629),r[""]()}}]);