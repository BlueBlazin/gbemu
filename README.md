# GBEmu

![cover](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/cover.png)

A Gameboy emulator written in Rust, compiled to WebAssembly, and running in the browser.

## Usage

Visit the website, load your ROM and begin playing! The emulator uses:

- **Arrow keys** for Up, Down, Left, and Right
- **A** and **S** for A and B
- **Enter** and **Space** for Start and Select.

## Screenshots

![3](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/3.png)
![1](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/1.png)
![2](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/2.png)
![4](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/4.png)
![5](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/5.png)
![6](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/6.png)
![7](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/7.png)
![8](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/8.png)
![9](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/9.png)
![10](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/10.png)
![11](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/11.png)
![12](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/12.png)
![13](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/13.png)
![14](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/14.png)
![15](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/15.png)
![16](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/16.png)
![17](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/17.png)
![18](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/18.png)
![19](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/19.png)
![20](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/20.png)
![21](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/21.png)
![22](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/22.png)
![23](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/23.png)
![24](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/24.png)
![25](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/25.png)
![26](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/26.png)
![27](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/27.png)

## Tests

![1](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/1.png)
![2](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/2.png)
![3](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/3.png)
![4](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/4.png)
![5](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/5.png)
![6](https://raw.githubusercontent.com/BlueBlazin/gbemu/master/screenshots/blargg/6.png)

## Attributions

I am grateful to the direct and indirect help of several people to make this emulator possible.

**SameBoy:** Initially my emulator didn't refer much to SameBoy but as accuracy became of interest I heavily copied some of SameBoy's way of doing things.

**Binjigb:** I used this as a reference for implementing some of the frontend logic. This includes an event based emulation driver, as well as some WASM stuff.

**WasmBoy:** I want to thank the torch2424, the creator of WasmBoy for initially suggesting using an event based emulation driver as well as offering advice on switching from RAF to setTimeout.

**gbdev:** I wouldn't have been able to complete the emulator without the daily help I got from members of the gbdev discord community.

**Pandocs, Blargg, Mooneye-gb, AntonioND:** These docs and tests were invaluable!
