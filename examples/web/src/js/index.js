// license:BSD-3-Clause
import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from "./wasi_wasmer";
import worklet from 'worklet:./wgm_worklet_processor.js';

/**
 * WebAssembly
 */
let memory;

/**
 * vgm setting
 */
const DEFAULT_SAMPLING_RATE = 44100;
const DEFAULT_SAMPLING_CHUNK = 2048;
const LOOP_MAX_COUNT = 2;
const FEED_OUT_SECOND = 2;

/**
 * canvas settings
 */
const CANVAS_WIDTH = 768;
const CANVAS_HEIGHT = 576;
const COLOR_MD_GREEN = '#00a040';
const COLOR_MD_RED = '#e60012';
const FONT_MAIN_STYLE = '16px sans-serif';

/**
 * vgm member
 */
let wgmplay = null;
let seqdata;
let playlist = [];
let totalPlaylistCount;
let musicMeta;
let samplingRate = DEFAULT_SAMPLING_RATE;
let samplingChunk;
let feedOutRemain

/**
 * audio contect
 */
let audioContext = null;
let audioNode = null;
let audioGain;
let audioAnalyser;
let audioAnalyserBuffer;
let audioAnalyserBufferLength;

/**
 * buffering
 */
const MAX_BUFFER_SIZE = 10;
let samplingBufferL;
let samplingBufferR;
let bufferd;
let playBufferPos;
let bufferdPos;
let feedOutPos
let bufferTimerId = null;

/**
 * canvas member
 */
let canvas;
let canvasContext;
let animId = null;

/**
 * canvas setting
 */
(function() {
    canvas = document.getElementById('screen');
    canvas.setAttribute('width', CANVAS_WIDTH);
    canvas.setAttribute('height', CANVAS_HEIGHT);
    let pixelRatio = window.devicePixelRatio ? window.devicePixelRatio : 1;
    if(pixelRatio > 1 && window.screen.width < CANVAS_WIDTH) {
        canvas.style.width = 320 + "px";
        canvas.style.heigth = 240 + "px";
    }
    canvasContext = canvas.getContext('2d');
})();

/**
 * Switch sampling rate for test (ex. https://.../#s=48000)
 *
 *  let context = new AudioContext({ sampleRate: samplingRate })
 *
 * (2021/9)
 * Support Firefox only. (I haven't confirmed anything other than Linux platform)
 * In other browsers, the setting works, but the native connection to the audio interface drops to 44100Hz.
 * There is probably some downsampling going on inside the browser.
 * Also, the setting itself may be invalid in Safari.
 */
(function() {
    if(location.hash != "") {
        const sample = location.hash.match(/^#s=(\d+)/);
        if(sample != null) {
            samplingRate = parseInt(sample[1]);
            if(samplingRate != samplingRate /* isNan */
                || !(samplingRate == 44100 || samplingRate == 48000 || samplingRate == 88200 || samplingRate == 96000)) {
                samplingRate = DEFAULT_SAMPLING_RATE;
            }
        }
    }
})();

/**
 * AudioContext Setting (TODO:)
 */
(function() {
    // hack for AudioContext opening is delayed on Linux.
    audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: samplingRate });
    const scriptNode = audioContext.createScriptProcessor(0, 2, 2);
    samplingChunk = scriptNode? scriptNode.bufferSize: DEFAULT_SAMPLING_CHUNK;
    audioContext = null;
    feedOutRemain = (samplingRate * FEED_OUT_SECOND) / samplingChunk;
})();

/**
 * load sample vgm data
 */
(async function() {
    // WebAssemby init
    const exports = await initWasi();
    setWasmExport(exports);
    memory = exports.memory;

    // load sample vgm
    const response = await fetch('./vgm/ym2612.vgm');
    const bytes = await response.arrayBuffer();

    // ready for sample vgm to play
    wgminit(bytes);

    // enable UI
    canvas.addEventListener('click', play, false);
    canvas.addEventListener('dragover', function(e) {
        prevent(e);
        canvas.style.border = '4px dotted #333333';
        return false;
    }, false);
    canvas.addEventListener('dragleave', function(e) {
        prevent(e);
        canvas.style.border = 'none';
        return false;
    });
    canvas.addEventListener('drop', onDrop, false);
    // for sample vgm
    totalPlaylistCount = 1;
    musicMeta = createGd3meta({
        track_name: "WebAssembly 👾 VGM Player",
        track_name_j: "",
        game_name: "",
        game_name_j: "YM2612 sample VGM",
        track_author: "@h1romas4",
        track_author_j: ""
    });
    // ready to go
    startScreen();
})();

/**
 * fill text center
 *
 * @param {*} str
 * @param {*} height
 */
const fillTextCenterd = function(str, height) {
    let left = (CANVAS_WIDTH - canvasContext.measureText(str).width) / 2;
    canvasContext.fillText(str, left, height);
}

/**
 * draw start screen
 */
const startScreen = function() {
    canvasContext.fillStyle = 'rgb(0, 0, 0)';
    canvasContext.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
    canvasContext.font = 'bold 28px sans-serif';
    canvasContext.fillStyle = COLOR_MD_RED;
    fillTextCenterd("WebAssembly 🎮 VGM Player", CANVAS_HEIGHT / 2 - 32 * 4);
    canvasContext.fillStyle = COLOR_MD_GREEN;
    canvasContext.font = '15px sans-serif';
    fillTextCenterd("YM2151 | YM2203 | YM2149 | YM2413 | YM2612 | SN76489(MD) | PWM(32x) | SEGAPCM", CANVAS_HEIGHT / 2 - 32 * 2);
    canvasContext.font = '20px sans-serif';
    fillTextCenterd("🎵 DRAG AND DROP VGM(vgm/vgz) HEAR", CANVAS_HEIGHT / 2 - 32 * 1);
    fillTextCenterd("OR CLICK(TAP) TO PLAY SAMPLE VGM", CANVAS_HEIGHT / 2 + 32 * 1);
    printStatus();
};

/**
 * event prevent
 *
 * @param {*} e
 */
const prevent = function(e) {
    e.preventDefault();
    e.stopPropagation();
};

/**
 * Drag and Drop
 *
 * @param {*} ev
 * @returns false (prevent event)
 */
const onDrop = function(ev) {
    prevent(ev);
    canvas.removeEventListener('drop', onDrop, false);
    canvas.style.border = 'none';
    let filelist = {};
    let files = ev.dataTransfer.files;
    [].forEach.call(files, function(file) {
        let reader = new FileReader();
        reader.onload = function() {
            filelist[file.name] = reader.result;
            if(Object.keys(filelist).length >= files.length) {
                canvas.addEventListener('drop', onDrop, false);
                playlist = [];
                Object.keys(filelist).sort().forEach(function(key) {
                    playlist.push(filelist[key]);
                });
                totalPlaylistCount = playlist.length;
                next();
            }
        };
        reader.readAsArrayBuffer(file);
    });
    return false;
};

/**
 * play next playlist
 */
const next = function() {
    if(playlist.length <= 0) return;
    if(wgminit(playlist.shift())) {
        play();
    } else {
        next();
    }
}

/**
 * create GD3 meta
 *
 * @param {*} meta
 * @returns
 */
const createGd3meta = function(meta) {
    meta.game_track_name = [meta.game_name, meta.track_name].filter(str => str != "").join(" | ");
    meta.game_track_name_j = [meta.game_name_j, meta.track_name_j].filter(str => str != "").join(" / ");
    meta.track_author_full = [meta.track_author, meta.track_author_j].filter(str => str != "").join(" - ");
    canvasContext.font = FONT_MAIN_STYLE;
    meta.game_track_name_left = (CANVAS_WIDTH - canvasContext.measureText(meta.game_track_name).width) / 2;
    meta.game_track_name_j_left = (CANVAS_WIDTH - canvasContext.measureText(meta.game_track_name_j).width) / 2;
    meta.track_author_full_left = (CANVAS_WIDTH - canvasContext.measureText(meta.track_author_full).width) / 2;
    return meta;
};

/**
 * wgminit
 *
 * @param ArrayBuffer vgmfile
 */
const wgminit = function(vgmfile) {
    if(wgmplay != null) wgmplay.free();
    // create wasm instanse
    wgmplay = new WgmPlay(samplingRate, samplingChunk, vgmfile.byteLength);
    // set vgmdata
    seqdata = new Uint8Array(memory.buffer, wgmplay.get_seq_data_ref(), vgmfile.byteLength);
    seqdata.set(new Uint8Array(vgmfile));
    // init player
    if(!wgmplay.init()) {
        wgmplay.free();
        wgmplay = null;
        return false;
    }

    musicMeta = createGd3meta(JSON.parse(wgmplay.get_seq_gd3()));

    // init buffer
    samplingBufferL = [];
    samplingBufferR = [];
    playBufferPos = 0;
    bufferdPos = 0;
    feedOutPos = null;

    if(bufferTimerId != null) {
        clearInterval(bufferTimerId);
        bufferTimerId = null;
    }

    let stop = false;
    let feedOutCount = 0;
    bufferd = false;
    bufferTimerId = setInterval(() => {
        if(stop) {
            clearInterval(bufferTimerId);
        }
        let loop;
        try {
            if(samplingBufferL.length < MAX_BUFFER_SIZE && !stop) {
                loop = wgmplay.play();
                // clone buffer
                let bufferL = new Float32Array(samplingChunk);
                let bufferR = new Float32Array(samplingChunk);
                bufferL.set(new Float32Array(memory.buffer, wgmplay.get_sampling_l_ref(), samplingChunk));
                bufferR.set(new Float32Array(memory.buffer, wgmplay.get_sampling_r_ref(), samplingChunk));
                samplingBufferL.push(bufferL);
                samplingBufferR.push(bufferR);
                if(samplingBufferL.length >= MAX_BUFFER_SIZE) {
                    bufferd = true;
                }
                if(loop >= LOOP_MAX_COUNT) {
                    if(feedOutCount == 0 && loop > LOOP_MAX_COUNT) {
                        // no loop track
                        stop = true;
                    } else {
                        // feedout loop track
                        if(feedOutCount == 0 ) {
                            feedOutPos = bufferdPos;
                        }
                        feedOutCount++;
                        if(feedOutCount > feedOutRemain) {
                            stop = true;
                        }
                    }
                }
                bufferdPos++;
            }
        } catch(e) {
            alert(`ymfm:\n\nAn unexpected error has occurred. System has stoped. Please reload brwoser.\n\n${e}`);
            stop = true;
        }
    }, samplingChunk / samplingRate / 2 * 1000);

    return true;
}

/**
 * disconnect
 */
const disconnect = function() {
    if(audioAnalyser != null) audioAnalyser.disconnect();
    if(audioGain != null) audioGain.disconnect();
    if(audioNode != null) audioNode.disconnect();
    // force GC
    audioAnalyser = null;
    audioNode = null;
    audioGain = null;
}

/**
 * play
 */
const play = async function() {
    canvas.removeEventListener('click', play, false);
    // recreate audio node for prevent memory leak.
    disconnect();
    // iOS only sounds AudioContext that created by the click event.
    if(audioContext == null) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: samplingRate });
    }
    // register audio worklet processor
    await audioContext.audioWorklet.addModule(worklet);
    audioNode = new AudioWorkletNode(audioContext, "wgm-worklet-processor");
    // register audioNode
    // audioNode = audioContext.createScriptProcessor(samplingChunk, 2, 2);
    // audioNode.onaudioprocess = (ev) => {
    //     if(bufferd == true && samplingBufferL.length > 0) {
    //         // set sampling buffer (re-attach view every cycle)
    //         ev.outputBuffer.getChannelData(0).set(samplingBufferL[0]);
    //         ev.outputBuffer.getChannelData(1).set(samplingBufferR[0]);
    //         samplingBufferL.shift();
    //         samplingBufferR.shift();
    //         if(playBufferPos == feedOutPos) {
    //             audioGain.gain.setValueAtTime(1, audioContext.currentTime);
    //             audioGain.gain.linearRampToValueAtTime(0, audioContext.currentTime + FEED_OUT_SECOND);
    //         }
    //         playBufferPos++;
    //     } else if (bufferd == true && samplingBufferL.length == 0) {
    //         disconnect();
    //         next();
    //     }
    // };
    // connect gain
    audioGain = audioContext.createGain();
    audioNode.connect(audioGain);
    audioGain.connect(audioContext.destination);
    audioGain.gain.setValueAtTime(1, audioContext.currentTime);
    // connect fft
    audioAnalyser = audioContext.createAnalyser();
    audioAnalyserBufferLength = audioAnalyser.frequencyBinCount;
    audioAnalyserBuffer = new Uint8Array(audioAnalyserBufferLength);
    audioAnalyser.getByteTimeDomainData(audioAnalyserBuffer);
    audioGain.connect(audioAnalyser);
    if(animId != null) {
        window.cancelAnimationFrame(animId);
        animId = null;
    }
    draw();
};

/**
 * draw
 */
const draw = function() {
    animId = window.requestAnimationFrame(draw);
    canvasContext.fillStyle = 'rgb(0, 0, 0)';
    canvasContext.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
    if(audioAnalyser != null) {
        audioAnalyser.getByteFrequencyData(audioAnalyserBuffer);

        canvasContext.lineWidth = 1;
        canvasContext.beginPath();
        canvasContext.strokeStyle = COLOR_MD_RED;

        let width = 4;
        let step =  Math.round(audioAnalyserBufferLength / (CANVAS_WIDTH / width));
        canvasContext.setLineDash([2, 1]);
        canvasContext.lineWidth = width ;
        for(var i = 0; i < audioAnalyserBufferLength; i += step) {
            canvasContext.beginPath();
            canvasContext.moveTo(i + 2, CANVAS_HEIGHT);
            canvasContext.lineTo(i + 2, CANVAS_HEIGHT - (audioAnalyserBuffer[i] * 1.5));
            canvasContext.stroke();
        }
        canvasContext.stroke();
    }

    canvasContext.font = "12px monospace";
    canvasContext.fillStyle = COLOR_MD_GREEN;
    if(totalPlaylistCount >= 1) {
        fillTextCenterd("Track " + (totalPlaylistCount - playlist.length) + " / " + totalPlaylistCount, CANVAS_HEIGHT / 2 - 96);
    }
    canvasContext.font = FONT_MAIN_STYLE;
    canvasContext.fillText(musicMeta.game_track_name, musicMeta.game_track_name_left, CANVAS_HEIGHT / 2 - 64);
    canvasContext.fillText(musicMeta.game_track_name_j, musicMeta.game_track_name_j_left, CANVAS_HEIGHT / 2 - 32);
    canvasContext.fillText(musicMeta.track_author_full, musicMeta.track_author_full_left, CANVAS_HEIGHT / 2);
    printStatus();
}

const printStatus = function() {
    if(samplingRate == 44100) return;

    const status = " HD:" + samplingRate + " ";
    canvasContext.font = '16px sans-serif';
    const measure = canvasContext.measureText(status);
    canvasContext.fillStyle = COLOR_MD_GREEN;
    canvasContext.fillRect(CANVAS_WIDTH - measure.width, 0, CANVAS_WIDTH, 18);
    canvasContext.fillStyle = 'rgb(0, 0, 0)';
    canvasContext.fillText(status, CANVAS_WIDTH - measure.width, 16);
}
