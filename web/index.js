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
  const oscillatorTypes = ['sine', 'square', 'sawtooth', 'triangle'];
  const duration = 5.0; // Duration in seconds

  statusEl.textContent = 'Initializing audio system...';

  for (const type of oscillatorTypes) {
    console.log(
      `\nSweeping BandlimitedWavetableOscillator with ${type} waveform...`
    );
    statusEl.textContent = `Sweeping BandlimitedWavetableOscillator with ${type}...`;

    // Wavetable oscillator sweep
    await handle.sweep_wavetable(type, 20.0, 10000.0, duration);
    console.log(`Started wavetable ${type} sweep`);

    // Wait slightly less than the full duration to account for fade out
    await sleep((duration - 0.01) * 1000);
    console.log('Finishing wavetable sweep');

    // Start fade out slightly before the end
    handle.silence_wavetable();
    await sleep(20); // Wait for fade out

    console.log(`\nSweeping Oscillator with ${type} waveform...`);
    statusEl.textContent = `Sweeping Oscillator with ${type}...`;

    // Regular oscillator sweep
    await handle.sweep_regular(type, 20.0, 10000.0, duration);
    console.log(`Started regular ${type} sweep`);

    // Wait slightly less than the full duration to account for fade out
    await sleep((duration - 0.01) * 1000);
    console.log('Finishing regular oscillator sweep');

    // Start fade out slightly before the end
    handle.silence_regular();
    await sleep(20); // Wait for fade out

    console.log(`Finished sweeping ${type} waveform for both oscillators.\n`);
    statusEl.textContent = `Finished sweeping ${type} waveform for both oscillators.`;
  }

  console.log('All sweeps completed!');
  statusEl.textContent = 'All sweeps completed!';
}

sweepButton.addEventListener('click', async () => {
  try {
    const wasm = await loadWasmModule();

    if (!handle) {
      console.log('Initializing audio system...');
      handle = wasm.Handle.new();
      await handle.start();
      console.log('Audio system initialized');
    }

    await performSweep();
  } catch (e) {
    console.error('Error during sweep:', e);
    statusEl.textContent = `Error during sweep: ${e.message}`;
  }
});
