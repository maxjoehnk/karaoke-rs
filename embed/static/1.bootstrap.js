(window.webpackJsonp=window.webpackJsonp||[]).push([[1],[,function(t,n,e){"use strict";e.r(n);var r=e(2),o=e(3);Object(r.s)();const c=300,i=210,u=1.5,s=.013333333,f=new window.AudioContext,a=f.createBufferSource(),d=document.getElementById("background"),l=document.getElementById("hidden"),h=l.getContext("2d");l.style.display="none",l.width=c,l.height=i;const w=document.getElementById("karaoke"),g=w.getContext("2d");function p(){w.width=document.documentElement.clientWidth,w.height=document.documentElement.clientHeight,console.log("Width:",w.width," Height:",w.height),v||b()}function b(){g.drawImage(d,0,0,1912,1077,0,0,w.width,w.height)}async function m(t){try{const n=await fetch(t),e=await n.arrayBuffer();if(!n.ok)throw new Error("Network response was not ok.");return e}catch(t){console.log("There has been a problem with your fetch operation: ",t.message)}}function y(t,n){f.decodeAudioData(t,(function(t){a.buffer=t,console.log("Song added to buffer...")}),(function(t){console.log("Error with decoding audio data"+t.err)})).then((function(){a.connect(f.destination),a.start(),async function(t){var n,e,r,o;for(;v;)n=f.currentTime,(e=Math.floor(n/s))>=0&&(o=e-r,t.next_frame(o),E(t)),r=e,await k(10);b()}(n)}))}function k(t){return new Promise(n=>setTimeout(n,t))}function E(t){let n=t.rainbow_cycle();g.fillStyle=`rgba(${n.r}, ${n.g}, ${n.b}, ${n.a})`,g.fillRect(0,0,w.width,w.height),g.fillStyle="black",g.fillRect(w.width/2-c*u/2,w.height/2-i*u/2,c*u,i*u);const e=t.frame(),r=new Uint8ClampedArray(o.s.buffer,e,c*i*4),s=new ImageData(r,c,i);h.putImageData(s,0,0),g.drawImage(l,0,0,c,i,w.width/2-c*u/2,w.height/2-i*u/2,c*u,i*u)}window.addEventListener("resize",p),p(),a.onended=()=>{v=!1,console.log("Song ended")},b();var v=!1;const _=window.location.hostname,I=new WebSocket("ws://"+_+":9090","rust-websocket");I.addEventListener("open",(function(t){I.send("Hello Server!")})),I.addEventListener("message",(function(t){console.log("Message from server:",t.data)})),I.addEventListener("close",(function(t){console.log("Socket closed")})),I.addEventListener("error",(function(t){console.log("Socket closed due to error:",t)})),async function(){try{const t=await fetch("/player/next"),n=await t.json();if(!t.ok)throw new Error("Network response was not ok.");return n.message}catch(t){console.log("There has been a problem with your fetch operation: ",t.message)}}().then((function(t){if(null!=t){console.log(t);var n="songs/"+t+".cdg";m("songs/"+t+".mp3").then((function(t){m(n).then((function(n){const e=r.a.new(n);y(t,e),v=!0}))}))}}))},function(t,n,e){"use strict";e.d(n,"s",(function(){return o})),e.d(n,"a",(function(){return y})),e.d(n,"j",(function(){return E})),e.d(n,"k",(function(){return v})),e.d(n,"m",(function(){return _})),e.d(n,"n",(function(){return I})),e.d(n,"o",(function(){return x})),e.d(n,"p",(function(){return S})),e.d(n,"q",(function(){return A})),e.d(n,"r",(function(){return j})),e.d(n,"e",(function(){return B})),e.d(n,"h",(function(){return T})),e.d(n,"c",(function(){return C})),e.d(n,"b",(function(){return L})),e.d(n,"d",(function(){return D})),e.d(n,"f",(function(){return O})),e.d(n,"g",(function(){return $})),e.d(n,"l",(function(){return H})),e.d(n,"i",(function(){return M}));var r=e(3);function o(){r.r()}const c=new Array(32);c.fill(void 0),c.push(void 0,null,!0,!1);let i=c.length;function u(t){i===c.length&&c.push(c.length+1);const n=i;return i=c[n],c[n]=t,n}function s(t){return c[t]}function f(t){const n=s(t);return function(t){t<36||(c[t]=i,i=t)}(t),n}let a=new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}),d=null;function l(){return null!==d&&d.buffer===r.s.buffer||(d=new Uint8Array(r.s.buffer)),d}function h(t,n){return a.decode(l().subarray(t,t+n))}let w=0,g=new TextEncoder("utf-8");const p="function"==typeof g.encodeInto?function(t,n){return g.encodeInto(t,n)}:function(t,n){const e=g.encode(t);return n.set(e),{read:t.length,written:e.length}};let b=null;function m(){return null!==b&&b.buffer===r.s.buffer||(b=new Int32Array(r.s.buffer)),b}class y{static __wrap(t){const n=Object.create(y.prototype);return n.ptr=t,n}free(){const t=this.ptr;this.ptr=0,r.a(t)}static new(t){const n=r.o(u(t));return y.__wrap(n)}next_frame(t){r.p(this.ptr,t)}frame(){return r.n(this.ptr)}rainbow_cycle(){const t=r.q(this.ptr);return k.__wrap(t)}}class k{static __wrap(t){const n=Object.create(k.prototype);return n.ptr=t,n}free(){const t=this.ptr;this.ptr=0,r.f(t)}get r(){return r.e(this.ptr)}set r(t){r.j(this.ptr,t)}get g(){return r.d(this.ptr)}set g(t){r.i(this.ptr,t)}get b(){return r.c(this.ptr)}set b(t){r.h(this.ptr,t)}get a(){return r.b(this.ptr)}set a(t){r.g(this.ptr,t)}}const E=function(t){f(t)},v=function(t,n){return u(h(t,n))},_=function(t,n,e,r){console.debug(s(t),s(n),s(e),s(r))},I=function(t){console.error(s(t))},x=function(t,n,e,r){console.error(s(t),s(n),s(e),s(r))},S=function(t,n,e,r){console.info(s(t),s(n),s(e),s(r))},A=function(t,n,e,r){console.log(s(t),s(n),s(e),s(r))},j=function(t,n,e,r){console.warn(s(t),s(n),s(e),s(r))},B=function(){return u(new Error)},T=function(t,n){const e=function(t){let n=t.length,e=r.l(n);const o=l();let c=0;for(;c<n;c++){const n=t.charCodeAt(c);if(n>127)break;o[e+c]=n}if(c!==n){0!==c&&(t=t.slice(c)),e=r.m(e,n,n=c+3*t.length);const o=l().subarray(e+c,e+n);c+=p(t,o).written}return w=c,e}(s(n).stack),o=w;m()[t/4+0]=e,m()[t/4+1]=o},C=function(t,n){const e=h(t,n).slice();r.k(t,1*n),console.error(e)},L=function(t){return u(s(t).buffer)},D=function(t){return s(t).length},O=function(t){return u(new Uint8Array(s(t)))},$=function(t,n,e){s(t).set(s(n),e>>>0)},H=function(t,n){throw new Error(h(t,n))},M=function(){return u(r.s)}},function(t,n,e){"use strict";var r=e.w[t.i];t.exports=r;e(2);r.t()}]]);