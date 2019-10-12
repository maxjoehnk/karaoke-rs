import { init, Cdg } from "web-player-wasm";
import { memory } from "web-player-wasm/web_player_wasm_bg";

init();

const cdg_width = 300;
const cdg_height = 210;
const cdg_scale = 1.5;
const sector_time = 0.013333333;
const audioCtx = new window.AudioContext;
const source = audioCtx.createBufferSource();

const background = document.getElementById("background");

// Hidden canvas for putting CDG image into
const hidden = document.getElementById("hidden");
const hiddenContext = hidden.getContext("2d");
hidden.style.display = "none";
hidden.width = cdg_width;
hidden.height = cdg_height;

// CDG Image will be drawn to this canvas
const canvas = document.getElementById("karaoke");
const canvasContext = canvas.getContext('2d');

// Keep canvas dimmensions updated with resizes & set to full window
function resizeCanvas() {
    canvas.width = document.documentElement.clientWidth;
    canvas.height = document.documentElement.clientHeight;
    console.log("Width:", canvas.width, " Height:", canvas.height);
    if (!song_playing) {
        drawBackground();
    }
}
window.addEventListener("resize", resizeCanvas);
resizeCanvas();

function drawBackground() {
    canvasContext.drawImage(
        background,
        0,
        0,
        1912,
        1077,
        0,
        0,
        canvas.width,
        canvas.height,
    );
}

// Get array buffer of file from path
async function getData(path) {
    try {
        const response = await fetch(path);
        const data = await response.arrayBuffer();
        if (!response.ok) {
            throw new Error('Network response was not ok.');
        }
        return data
    }
    catch (error) {
        console.log('There has been a problem with your fetch operation: ', error.message);
    }
}

// Get array buffer of file from path
async function nextSongName() {
    try {
        const response = await fetch("/player/next");
        const data = await response.json();
        if (!response.ok) {
            throw new Error('Network response was not ok.');
        }
        return data.message
    }
    catch (error) {
        console.log('There has been a problem with your fetch operation: ', error.message);
    }
}

function play(audioData, cdg) {
    audioCtx.decodeAudioData(audioData, function (buffer) {
        source.buffer = buffer;
        console.log("Song added to buffer...");
    },
        function (e) {
            console.log("Error with decoding audio data" + e.err);
        }).then(function () {
            source.connect(audioCtx.destination);
            source.start();
            demo(cdg);
        });
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function render(cdg) {
    let rgba = cdg.rainbow_cycle();
    canvasContext.fillStyle = `rgba(${rgba.r}, ${rgba.g}, ${rgba.b}, ${rgba.a})`;
    canvasContext.fillRect(0, 0, canvas.width, canvas.height);
    canvasContext.fillStyle = "black";
    canvasContext.fillRect(
        canvas.width / 2 - (cdg_width * cdg_scale) / 2,
        canvas.height / 2 - (cdg_height * cdg_scale) / 2,
        cdg_width * cdg_scale,
        cdg_height * cdg_scale
    );

    const framePtr = cdg.frame();
    const frameBuffer = new Uint8ClampedArray(memory.buffer, framePtr, cdg_width * cdg_height * 4);
    const imageData = new ImageData(frameBuffer, cdg_width, cdg_height);
    hiddenContext.putImageData(imageData, 0, 0);
    canvasContext.drawImage(
        hidden,
        0,
        0,
        cdg_width,
        cdg_height,
        canvas.width / 2 - (cdg_width * cdg_scale) / 2,
        canvas.height / 2 - (cdg_height * cdg_scale) / 2,
        cdg_width * cdg_scale,
        cdg_height * cdg_scale,
    );
}

async function demo(cdg) {
    var track_pos;
    var calc_sector;
    var last_sector;
    var sectors_since;

    while (song_playing) {
        track_pos = audioCtx.currentTime;
        calc_sector = Math.floor(track_pos / sector_time);

        if (calc_sector >= 0) {
            sectors_since = calc_sector - last_sector;

            cdg.next_frame(sectors_since);
            render(cdg);
        }

        last_sector = calc_sector;
        await sleep(10);
    }

    drawBackground();
}

source.onended = () => {
    song_playing = false;
    console.log("Song ended");
}

drawBackground();
var song_playing = false;

function run() {
    nextSongName().then(function (song) {
        if (song != null) {

            console.log(song);
            var mp3_file = "songs/" + song + ".mp3";
            var cdg_file = "songs/" + song + ".cdg";

            getData(mp3_file).then(function (audioData) {
                getData(cdg_file).then(function (cdgData) {
                    const cdg = Cdg.new(cdgData);

                    play(audioData, cdg);
                    song_playing = true;
                });
            });
        }
    });
}

// Create WebSocket connection.
const host = window.location.hostname;
const socket = new WebSocket('ws://' + host + ':9090', 'rust-websocket');

// Connection opened
socket.addEventListener('open', function (event) {
    socket.send('Hello Server!');
});

// Listen for messages
socket.addEventListener('message', function (event) {
    console.log('Message from server:', event.data);
});

// Connection closed
socket.addEventListener('close', function (event) {
    console.log('Socket closed');
});

// Connection error
socket.addEventListener('error', function (event) {
    console.log('Socket closed due to error:', event);
});

// run();
