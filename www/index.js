import { Emulation } from "./emulation";

const emulation = new Emulation();

const inputElement = document.getElementById("input");
inputElement.addEventListener("change", handleFiles, false);

function handleFiles() {
  const romFile = this.files[0];

  romFile.arrayBuffer().then((buffer) => {
    emulation.start(new Uint8Array(buffer));
  });
}
