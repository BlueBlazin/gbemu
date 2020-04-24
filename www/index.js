import { Emulator } from "gbemu";
import { memory } from "gbemu/gbemu_bg";

const WIDTH = 160;
const HEIGHT = 144;
const CHANNELS = 4;
const PIXEL_SIZE = 1;

/*********************************************************
 *  Audio (Experimental)
 **********************************************************/

/*********************************************************
 *  Canvas
 **********************************************************/

const canvas = document.getElementById("gbemu-canvas");
canvas.width = PIXEL_SIZE * WIDTH;
canvas.height = PIXEL_SIZE * HEIGHT;
const ctx = canvas.getContext("2d", { alpha: false });

/*********************************************************
 *  Graphics
 **********************************************************/

function drawScreen(screenPtr) {
  const screen = new Uint8ClampedArray(
    memory.buffer,
    screenPtr,
    WIDTH * HEIGHT * CHANNELS
  );

  const image = new ImageData(screen, WIDTH, HEIGHT);

  ctx.putImageData(image, 0, 0, 0, 0, WIDTH, HEIGHT);
  // ctx.drawImage(image, 0, 0, WIDTH * 2, HEIGHT * 2);
}

/*********************************************************
 *  Emulation
 ***********************************************************/

let keydownHandler;
let keyupHandler;

function emulate(romData) {
  const gb = Emulator.new();
  gb.load(romData);

  keydownHandler = window.addEventListener("keydown", (event) => {
    switch (event.keyCode) {
      case 39: {
        gb.keydown(0);
        break;
      }
      case 37: {
        gb.keydown(1);
        break;
      }
      case 38: {
        gb.keydown(2);
        break;
      }
      case 40: {
        gb.keydown(3);
        break;
      }
      case 65: {
        gb.keydown(4);
        break;
      }
      case 83: {
        gb.keydown(5);
        break;
      }
      case 32: {
        gb.keydown(6);
        break;
      }
      case 13: {
        gb.keydown(7);
        break;
      }
    }
  });

  keyupHandler = window.addEventListener("keyup", (event) => {
    switch (event.keyCode) {
      case 39: {
        gb.keyup(0);
        break;
      }
      case 37: {
        gb.keyup(1);
        break;
      }
      case 38: {
        gb.keyup(2);
        break;
      }
      case 40: {
        gb.keyup(3);
        break;
      }
      case 65: {
        gb.keyup(4);
        break;
      }
      case 83: {
        gb.keyup(5);
        break;
      }
      case 32: {
        gb.keyup(6);
        break;
      }
      case 13: {
        gb.keyup(7);
        break;
      }
    }
  });

  const screenPtr = gb.screen();

  function renderLoop() {
    // Update emulator
    gb.update();
    // Draw Screen
    drawScreen(screenPtr);
    requestAnimationFrame(renderLoop);
  }

  renderLoop();
}

const inputElement = document.getElementById("input");
inputElement.addEventListener("change", handleFiles, false);
function handleFiles() {
  const romFile = this.files[0];
  romFile.arrayBuffer().then((buffer) => {
    emulate(new Uint8Array(buffer));
  });
}
