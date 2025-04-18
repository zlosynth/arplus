# TODO

- [ ] Implement CV calibration on all CVs. So it can be used with tonic remapped
- [ ] Make sure there are no panics / unwraps
- [ ] Write build manual
- [ ] Write user manual
- [ ] Narrow down the amount of features - reduce number of chords.
- [ ] Refactor the whole codebase.
- [ ] Document why is RSNX interesting - polyrhythms, partial fix and random, ...
- [ ] Focus during testing on display changes that were not querried by pot movement or trigger.
- [ ] Add known issues to the manual - Achordino may have issues tracking.
  - [ ] With quantized Achordion (try the build with equal quantization)
  - [ ] With unquantized Achordion solo voice
- [ ] Add frequency on output measurement to the test plan.
- [ ] Release Achordion with even quantization as the default. With a note that alternative firmware with better accuracy for white keys is available.
- [ ] Handle accurate tuning in v1. In v1.1 focus on synchronization with Achordion:
  - Enahnced Achordion and Arplus calibration, mapping full CV range from 0 to 5 V. Backwards compatible - allow tracking a sequence of Cs, calculate the curve from them
  - Automated sync mode. When enabled, Arplus will configure scale of Achordion
  - Configuration option to send last root to quant output
  - Use bootloader on Achordion, to provide more space for firmware
  - Reuse scale libraries of Arplus on Achordion, if used together, Achordion can
    run in controlled mode.
  - When Arplus is configured that way, Achordion connected through Quant would detect that
    it is controlled by Arplus and enter controlled mode - scale would be following
    Arplus.
