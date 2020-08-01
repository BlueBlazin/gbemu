import { Emulator } from "gbemu";
import { memory } from "gbemu/gbemu_bg";

const WIDTH = 160;
const HEIGHT = 144;
const CHANNELS = 4;

const EVENT_VBLANK = 0;
const EVENT_AUDIO_BUFFER_FULL = 1;
const EVENT_MAX_CYCLES = 2;
const PIXEL_SIZE = 1;

// const AUDIO_BUFFER_SIZE = 736;
const AUDIO_BUFFER_SIZE = 2048;
const AUDIO_SAMPLE_RATE = 44100.0;
const SAMPLE_DURATION = AUDIO_BUFFER_SIZE / AUDIO_SAMPLE_RATE;

/*********************************************************
 *  Canvas
 **********************************************************/

const canvas = document.getElementById("gbemu-canvas");
canvas.width = PIXEL_SIZE * WIDTH;
canvas.height = PIXEL_SIZE * HEIGHT;
const ctx = canvas.getContext("2d", { alpha: false });

ctx.fillStyle = "#6a737b";
ctx.fillRect(0, 0, canvas.width, canvas.height);

/*********************************************************
 *  Audio
 **********************************************************/

const audioCtx = new (window.AudioContext || window.webkitAudioContext)();

/*********************************************************
 *  Emulation
 **********************************************************/

export class Emulation {
  start(romData) {
    this.gb = Emulator.new(romData);
    this.screenPtr = this.gb.screen();
    this.audioLeftPtr = this.gb.audio_buffer_left();
    this.audioRightPtr = this.gb.audio_buffer_right();

    this.registerKeydownHandler();
    this.registerKeyupHandler();

    this.lastCallTime = null;

    this.emulationDriver();
  }

  emulationDriver(time) {
    requestAnimationFrame(this.emulationDriver.bind(this));
    // setTimeout(this.emulationDriver.bind(this), 1000 / 60);

    const diff = time - this.lastCallTime;
    this.lastCallTime = time;

    const timeDelta = diff - 1000 / 60;

    let before = performance.now();
    this.runTill();
    console.log(performance.now() - before);
  }

  runTill() {
    let event;

    while (true) {
      event = this.gb.run_till_event();

      if (event == EVENT_VBLANK) {
        this.drawScreen();
      }

      if (event == EVENT_AUDIO_BUFFER_FULL) {
        this.playAudio();
      }

      if (event == EVENT_MAX_CYCLES) {
        break;
      }
    }
  }

  drawScreen() {
    const screen = new Uint8ClampedArray(
      memory.buffer,
      this.screenPtr,
      WIDTH * HEIGHT * CHANNELS
    );

    const image = new ImageData(screen, WIDTH, HEIGHT);
    ctx.putImageData(image, 0, 0, 0, 0, WIDTH, HEIGHT);
  }

  playAudio() {
    const leftBuffer = new Float32Array(
      memory.buffer,
      this.audioLeftPtr,
      AUDIO_BUFFER_SIZE
    );

    const rightBuffer = new Float32Array(
      memory.buffer,
      this.audioRightPtr,
      AUDIO_BUFFER_SIZE
    );

    const audioArrayBuffer = audioCtx.createBuffer(
      2,
      AUDIO_BUFFER_SIZE,
      AUDIO_SAMPLE_RATE
    );

    audioArrayBuffer.copyToChannel(leftBuffer, 0);
    audioArrayBuffer.copyToChannel(rightBuffer, 1);

    const audioSource = audioCtx.createBufferSource();
    audioSource.buffer = audioArrayBuffer;

    let startTime = this.nextStartTime || audioCtx.currentTime;

    audioSource.connect(audioCtx.destination);
    audioSource.start(startTime);

    this.nextStartTime = startTime + SAMPLE_DURATION;
  }

  registerKeydownHandler() {
    window.addEventListener("keydown", (event) => {
      switch (event.keyCode) {
        case 39: {
          this.gb.keydown(0);
          break;
        }
        case 37: {
          this.gb.keydown(1);
          break;
        }
        case 38: {
          this.gb.keydown(2);
          break;
        }
        case 40: {
          this.gb.keydown(3);
          break;
        }
        case 65: {
          this.gb.keydown(4);
          break;
        }
        case 83: {
          this.gb.keydown(5);
          break;
        }
        case 32: {
          this.gb.keydown(6);
          break;
        }
        case 13: {
          this.gb.keydown(7);
          break;
        }
      }
    });
  }

  registerKeyupHandler() {
    window.addEventListener("keyup", (event) => {
      switch (event.keyCode) {
        case 39: {
          this.gb.keyup(0);
          break;
        }
        case 37: {
          this.gb.keyup(1);
          break;
        }
        case 38: {
          this.gb.keyup(2);
          break;
        }
        case 40: {
          this.gb.keyup(3);
          break;
        }
        case 65: {
          this.gb.keyup(4);
          break;
        }
        case 83: {
          this.gb.keyup(5);
          break;
        }
        case 32: {
          this.gb.keyup(6);
          break;
        }
        case 13: {
          this.gb.keyup(7);
          break;
        }
      }
    });
  }
}
