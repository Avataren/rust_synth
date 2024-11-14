const init = import('./pkg').then((module) => {
  let handle = null;
  const sweepButton = document.getElementById('runSweep');
  const statusEl = document.getElementById('status');

  async function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function performFrequencySweep(setFrequency, duration = 5000) {
    const startFreq = 20;
    const endFreq = 10000;
    const startTime = Date.now();

    while (Date.now() - startTime < duration) {
      const progress = (Date.now() - startTime) / duration;
      const currentFreq = startFreq * Math.pow(endFreq / startFreq, progress);
      setFrequency(currentFreq);
      await new Promise((r) => requestAnimationFrame(r));
    }
  }

  async function performSweep() {
    const types = ['sine', 'square', 'sawtooth', 'triangle'];

    for (const type of types) {
      statusEl.textContent = `Sweeping BandlimitedWavetableOscillator with ${type}...`;
      await handle.sweep_wavetable(type);
      await performFrequencySweep((f) => handle.set_wavetable_frequency(f));
      handle.silence_wavetable();
      await sleep(500);

      statusEl.textContent = `Sweeping Oscillator with ${type}...`;
      await handle.sweep_regular(type);
      await performFrequencySweep((f) => handle.set_regular_frequency(f));
      handle.silence_regular();
      await sleep(500);

      statusEl.textContent = `Finished sweeping ${type} waveform for both oscillators.`;
    }

    statusEl.textContent = 'All sweeps completed!';
  }

  sweepButton.addEventListener('click', async () => {
    try {
      if (!handle) {
        handle = module.Handle.new();
        await handle.start();
      }

      await performSweep();
    } catch (e) {
      console.error(e);
      statusEl.textContent = 'Error during sweep';
    }
  });
});

init.catch(console.error);
