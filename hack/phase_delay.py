#!/usr/bin/env python
#
# An attempt to compensate for phase delay introduced by state variable filter.

import argparse
import math
import sys

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.widgets import Slider


def generate_sine(frequency, length, sample_rate):
    time = np.linspace(0, length, int(length * sample_rate))
    return np.sin(frequency * 2 * np.pi * time)


def low_pass_filter(data, bandlimit, q_factor, sample_rate):
    f = 2.0 * math.sin((math.pi * bandlimit) / sample_rate)
    q = 1.0 / q_factor
    delay_1 = 0.0
    delay_2 = 0.0

    filtered = np.zeros(len(data))

    for i in range(len(data)):
        sum_3 = delay_1 * f + delay_2
        sum_1 = data[i] - sum_3 - delay_1 * q
        sum_2 = sum_1 * f + delay_1
        delay_1 = sum_2
        delay_2 = sum_3
        filtered[i] = sum_3

    return filtered


slider_left = 0.05


def add_slider(fig, name, init, valmin, valmax):
    SLIDER_BOTTOM = 0.15
    SLIDER_WIDTH = 0.0225
    SLIDER_HEIGHT = 1.0 - SLIDER_BOTTOM * 2
    SLIDER_MARGIN = 0.03

    global slider_left
    slider = Slider(
        ax=fig.add_axes([slider_left, SLIDER_BOTTOM, SLIDER_WIDTH, SLIDER_HEIGHT]),
        label=name,
        valmin=valmin,
        valmax=valmax,
        valinit=init,
        orientation="vertical",
    )
    slider_left += SLIDER_MARGIN

    return slider


def cmd_sliders():
    SAMPLE_RATE = 48000
    INIT_FREQUENCY = 3.0
    INIT_CUTOFF = 1.2
    INIT_Q = 1.0

    fig, ax = plt.subplots()

    fig.subplots_adjust(left=0.25)

    frequency_slider = add_slider(fig, "Freq", INIT_FREQUENCY, 0.1, 40.0)
    cutoff_slider = add_slider(fig, "Filter", INIT_CUTOFF, 0.0, 2.0)
    q_slider = add_slider(fig, "Q", INIT_Q, 0.0, 2.0)

    def update(_):
        ax.cla()
        ax.set_ylim([-1.3, 1.3])
        ax.grid(True, axis="y")
        time = 4 / frequency_slider.val

        (line,) = ax.plot(np.zeros(int(SAMPLE_RATE * time)))
        input = generate_sine(frequency_slider.val, time, SAMPLE_RATE)
        line.set_ydata(input)

        (line,) = ax.plot(np.zeros(int(SAMPLE_RATE * time)))
        filtered = low_pass_filter(input, cutoff_slider.val * frequency_slider.val, q_slider.val, SAMPLE_RATE)
        line.set_ydata(filtered)

        fig.canvas.draw_idle()

    update(())

    frequency_slider.on_changed(update)
    cutoff_slider.on_changed(update)
    q_slider.on_changed(update)

    plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog=sys.argv[0])
    subparsers = parser.add_subparsers(
        help="sub-command help", required=True, dest="subparser"
    )
    subparsers.add_parser("sliders", help="Adjust parameters and see the delay")
    args = parser.parse_args()

    if args.subparser == "response":
        cmd_sliders()
