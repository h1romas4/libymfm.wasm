import { WgmPlay } from "../wasm/libymfm";
import { memory } from "../wasm/libymfm_bg.wasm";

/**
 * vgm setting
 */
const MAX_SAMPLING_BUFFER = 2048;
const SAMPLING_RATE = 44100;
const LOOP_MAX_COUNT = 2;
const FEED_OUT_SECOND = 2;
const FEED_OUT_REMAIN = (SAMPLING_RATE * FEED_OUT_SECOND) / MAX_SAMPLING_BUFFER;

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
let feedOutCount = 0;
let playlist = [];
let totalPlaylistCount;
let music_meta;

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
 * canvas member
 */
let canvas;
let canvasContext;
let animId = null;

/**
 * canvas setting
 */
canvas = document.getElementById('screen');
canvas.setAttribute('width', CANVAS_WIDTH);
canvas.setAttribute('height', CANVAS_HEIGHT);
let pixelRatio = window.devicePixelRatio ? window.devicePixelRatio : 1;
if(pixelRatio > 1 && window.screen.width < CANVAS_WIDTH) {
    canvas.style.width = 320 + "px";
    canvas.style.heigth = 240 + "px";
}
canvasContext = canvas.getContext('2d');

/**
 * load sample vgm data
 */
fetch('./vgm/ym2612.vgm')
    .then(response => response.arrayBuffer())
    .then(bytes => { init(bytes); })
    .then(() => {
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
        music_meta = createGd3meta({
            track_name: "WebAssembly 👾 VGM Player",
            track_name_j: "",
            game_name: "",
            game_name_j: "YM2612 sample VGM",
            track_author: "@h1romas4",
            track_author_j: ""
        });
        // hack for AudioContext opening is delayed on Linux.
        audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: SAMPLING_RATE });
        audioContext = null;
        // ready to go
        startScreen();
    });

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
    if(init(playlist.shift())) {
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
 * init
 *
 * @param ArrayBuffer vgmfile
 */
const init = function(vgmfile) {
    if(wgmplay != null) wgmplay.free();
    // create wasm instanse
    wgmplay = new WgmPlay(SAMPLING_RATE, MAX_SAMPLING_BUFFER, vgmfile.byteLength);
    // set vgmdata
    seqdata = new Uint8Array(memory.buffer, wgmplay.get_seq_data_ref(), vgmfile.byteLength);
    seqdata.set(new Uint8Array(vgmfile));
    // init player
    if(!wgmplay.init()) {
        wgmplay.free();
        return false;
    }

    music_meta = createGd3meta(JSON.parse(wgmplay.get_seq_gd3()));

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
const play = function() {
    canvas.removeEventListener('click', play, false);
    // recreate audio node for prevent memory leak.
    disconnect();
    // iOS only sounds AudioContext that created by the click event.
    if(audioContext == null) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: SAMPLING_RATE });
    }
    audioNode = audioContext.createScriptProcessor(MAX_SAMPLING_BUFFER, 2, 2);
    feedOutCount = 0;
    let stop = false;
    audioNode.onaudioprocess = function(ev) {
        // flash last sampling
        if(stop) {
            disconnect();
            next();
            return;
        }
        let loop;
        try {
            loop = wgmplay.play();
        } catch(e) {
            alert(`ymfm:\n\nAn unexpected error has occurred. System has stoped. Please reload brwoser.\n\n${e}`);
            stop = true;
        }
        // set sampling buffer (re-attach view every cycle)
        ev.outputBuffer.getChannelData(0).set(new Float32Array(memory.buffer, wgmplay.get_sampling_l_ref(), MAX_SAMPLING_BUFFER));
        ev.outputBuffer.getChannelData(1).set(new Float32Array(memory.buffer, wgmplay.get_sampling_r_ref(), MAX_SAMPLING_BUFFER));
        if(loop >= LOOP_MAX_COUNT) {
            if(feedOutCount == 0 && loop > LOOP_MAX_COUNT) {
                // no loop track
                stop = true;
            } else {
                // feedout loop track
                if(feedOutCount == 0 ) {
                    audioGain.gain.setValueAtTime(1, audioContext.currentTime);
                    audioGain.gain.linearRampToValueAtTime(0, audioContext.currentTime + FEED_OUT_SECOND);
                }
                feedOutCount++;
                if(feedOutCount > FEED_OUT_REMAIN) {
                    stop = true;
                }
            }
        }
    };
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
    canvasContext.fillText(music_meta.game_track_name, music_meta.game_track_name_left, CANVAS_HEIGHT / 2 - 64);
    canvasContext.fillText(music_meta.game_track_name_j, music_meta.game_track_name_j_left, CANVAS_HEIGHT / 2 - 32);
    canvasContext.fillText(music_meta.track_author_full, music_meta.track_author_full_left, CANVAS_HEIGHT / 2);
}
