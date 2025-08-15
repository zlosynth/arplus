# Changelog

All notable changes to this project will be documented in this file. See
[VERSIONING.md](VERSIONING.md) for more information about versioning and
backwards compatibility.

## Unreleased

* Add support for haas and ping pong output modes, with haas as the default.
* Tweak resonance range to be responsive end to end.
* Change contour range to 0.0 to 0.4 seconds linear up to 12 o'clock and
  0.4 to 10 logarithmic above that.
* Rework the interface to provide more direct control over attributes.
* Make it possible to offset scale steps by up to 4 quarter notes in either
  direction.
* Allow using a custom "noise" burst source.
* Make it possible to control the number of strummed strings.
* Persist the last selected scale per group.

## 0.1.1

* Fix ARP diagrams in the manual.
* Fix filter instability with long contour and high cutoff.

## 0.1.0

* The module is good to go.
