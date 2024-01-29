#!/usr/bin/env python3

A = 440.0
RANGE_IN_QUARTERTONES = (-138, 103)

for quarternotes in range(*RANGE_IN_QUARTERTONES):
    print("{:.8f}".format(A * 2.0 ** (quarternotes / 24.0)))
