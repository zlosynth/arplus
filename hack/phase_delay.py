#!/usr/bin/env python
#
# An attempt to compensate for phase delay introduced by state variable filter.

import argparse
import math
import sys

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.widgets import Slider

CUTOFF_INIT = 2.0
CUTOFF_MIN = 1.5
CUTOFF_MAX = 11.5
Q_INIT = 0.7
Q_MIN = 0.5
Q_MAX = 1.0

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
    INIT_FREQUENCY = 1.0

    fig, ax = plt.subplots()

    fig.subplots_adjust(left=0.25)

    frequency_slider = add_slider(fig, "Freq", INIT_FREQUENCY, 0.1, 40.0)
    cutoff_slider = add_slider(fig, "Filter", CUTOFF_INIT, CUTOFF_MIN, CUTOFF_MAX)
    q_slider = add_slider(fig, "Q", Q_INIT, Q_MIN, Q_MAX)

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

        input_zero_crossings = np.where(np.diff(np.sign(input)))[0]
        filtered_zero_crossings = np.where(np.diff(np.sign(filtered)))[0]
        fifth_input_zero_crossing_in_seconds = input_zero_crossings[4] / SAMPLE_RATE;
        fifth_filtered_zero_crossing_in_seconds = filtered_zero_crossings[4] / SAMPLE_RATE;
        delay_in_seconds = fifth_filtered_zero_crossing_in_seconds - fifth_input_zero_crossing_in_seconds;
        interval = 1.0 / frequency_slider.val;
        relative_delay = delay_in_seconds / interval;
        print("Delay", relative_delay)

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

    if args.subparser == "sliders":
        cmd_sliders()
