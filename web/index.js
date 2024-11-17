// index.js
let wasmModule = null;
let handle = null;
const sweepButton = document.getElementById('runSweep');
const statusEl = document.getElementById('status');

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function loadWasmModule() {
  if (!wasmModule) {
    wasmModule = await import('./pkg');
  }
  return wasmModule;
}

async function performSweep() {
  const types = ['sine', 'square', 'sawtooth', 'triangle'];
  const duration = 5.0; // Duration in seconds

  for (const type of types) {
    statusEl.textContent = `Sweeping BandlimitedWavetableOscillator with ${type}...`;
    await handle.sweep_wavetable(type, 20.0, 20000.0, duration);
    await sleep(duration * 1000);
    handle.silence_wavetable();
    await sleep(500);

    statusEl.textContent = `Sweeping Oscillator with ${type}...`;
    await handle.sweep_regular(type, 20.0, 20000.0, duration);
    await sleep(duration * 1000);
    handle.silence_regular();
    await sleep(500);

    statusEl.textContent = `Finished sweeping ${type} waveform for both oscillators.`;
  }

  statusEl.textContent = 'All sweeps completed!';
}

sweepButton.addEventListener('click', async () => {
  try {
    const wasm = await loadWasmModule();

    if (!handle) {
      handle = wasm.Handle.new();
      await handle.start();
    }

    await performSweep();
  } catch (e) {
    console.error(e);
    statusEl.textContent = 'Error during sweep';
  }
});
