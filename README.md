# Color Bomber

A portrait bomb-catching game. Purple, orange, and green bombs fall over a floating skyline. Match the lowest bomb of a color before it crosses the rooftops. A miss costs a life and damages the nearest building. The run ends at zero lives.

The playfield is 720×1080. The desktop window is the same shape at 480×720. There is no audio.

## Play

| Action | Desktop | Browser |
| --- | --- | --- |
| Start or restart | Space, or click | Tap, or Space |
| Purple | A, or the left button | Tap the left button |
| Orange | S, or the middle button | Tap the middle button |
| Green | D, or the right button | Tap the right button |

You start with 5 lives. Bombs fall faster and arrive sooner as the score climbs. The white center tower is scenery: it can burn, and it is not a fourth button.

## Desktop

```bash
cargo run
cargo test
```

## Browser

The site is a Workers static-asset deploy. The live build is [color-bomber.rafael-abuawad.workers.dev](https://color-bomber.rafael-abuawad.workers.dev).

One-time setup:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
npm install
cf auth login
```

`wasm-bindgen` 0.2.129 matches the version in `Cargo.lock`. `wasm-opt` (Binaryen) is optional; without it the script still builds, and the `.wasm` file may pass the 25 MiB Workers limit only after optimization.

Publish:

```bash
./scripts/build-web.sh
cf deploy
```

`scripts/build-web.sh` compiles the `wasm` profile, runs `wasm-bindgen`, copies `assets/sprites` and `assets/fonts` into `dist/`, and refuses to continue if the `.wasm` file is over 25 MiB. `cf deploy` uploads that directory. It does not run the script for you.

## Art and font

Runtime sprites live in `assets/sprites`. The Inkscape sources and reference screenshots are the other files under `assets/`. The on-screen font is [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P), under the SIL Open Font License (`assets/fonts/OFL.txt`).
