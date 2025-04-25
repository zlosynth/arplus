#!/usr/bin/env python
#
# An attempt to calcuate phase delay introduced by state variable filter.
# This is important for the karplus strong algorithm, since the LPF on
# its feedback loop extends the delay time and by that makes the output
# flatter than it's support to be.

import argparse
import math
import sys
import concurrent.futures
from functools import partial
from csv import DictWriter

import numpy as np
import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.widgets import Slider
from scipy.optimize import curve_fit


DELAY_DATASET = "delay_dataset.csv"
SAMPLE_RATE = 48_000
CUTOFF_INIT = 2.0
CUTOFF_MIN = 1.5
CUTOFF_MAX = 150.0
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
    INIT_FREQUENCY = 1.0

    fig, ax = plt.subplots()

    fig.subplots_adjust(left=0.25)

    frequency_slider = add_slider(fig, "Freq", 220, 20, 880)
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
        filtered = low_pass_filter(
            input, cutoff_slider.val * frequency_slider.val, q_slider.val, SAMPLE_RATE
        )
        line.set_ydata(filtered)

        input_zero_crossings = np.where(np.diff(np.sign(input)))[0]
        filtered_zero_crossings = np.where(np.diff(np.sign(filtered)))[0]
        fifth_input_zero_crossing_in_seconds = input_zero_crossings[4] / SAMPLE_RATE
        fifth_filtered_zero_crossing_in_seconds = (
            filtered_zero_crossings[4] / SAMPLE_RATE
        )
        delay_in_seconds = (
            fifth_filtered_zero_crossing_in_seconds
            - fifth_input_zero_crossing_in_seconds
        )
        interval = 1.0 / frequency_slider.val
        relative_delay = delay_in_seconds / interval
        print("Delay", relative_delay)

        fig.canvas.draw_idle()

    update(())

    frequency_slider.on_changed(update)
    cutoff_slider.on_changed(update)
    q_slider.on_changed(update)

    plt.show()


def set_relative_delay(config, frequency=50):
    time = 3 / frequency

    input = generate_sine(frequency, time, SAMPLE_RATE)
    filtered = low_pass_filter(input, config["c"] * frequency, config["q"], SAMPLE_RATE)

    input_zero_crossings = np.where(np.diff(np.sign(input)))[0]
    filtered_zero_crossings = np.where(np.diff(np.sign(filtered)))[0]

    fifth_input_zero_crossing_in_seconds = input_zero_crossings[4] / SAMPLE_RATE
    fifth_filtered_zero_crossing_in_seconds = filtered_zero_crossings[4] / SAMPLE_RATE

    delay_in_seconds = (
        fifth_filtered_zero_crossing_in_seconds - fifth_input_zero_crossing_in_seconds
    )
    interval = 1.0 / frequency
    config["d"] = delay_in_seconds / interval

    return config


def cmd_generate():
    C = 1000
    Q = 100

    input_configs = []
    for c in np.linspace(CUTOFF_MIN, CUTOFF_MAX, C):
        for q in np.linspace(Q_MIN, Q_MAX, Q):
            input_configs.append(
                {
                    "c": c,
                    "q": q,
                }
            )

    i = 1
    m = C * Q
    configs = []
    with concurrent.futures.ProcessPoolExecutor() as executor:
        for config in executor.map(set_relative_delay, input_configs):
            configs.append(config)
            print(f"{i}/{m}")
            i += 1

    with open(DELAY_DATASET, "w", newline="") as f:
        writer = DictWriter(f, fieldnames=configs[0].keys())
        writer.writeheader()
        writer.writerows(configs)

    print("Done")


def cmd_lookup():
    print("// ./hack/phase_delay.py lookup > dsp/src/phase_delay_lookup.rs && rustfmt dsp/src/phase_delay_lookup.rs")
    generate_lookup_table(
        index=1,
        table_c=64,
        table_c_min=1.3,
        table_c_max=10.0,
        table_q=8,
        table_q_min=0.5,
        table_q_max=1.0,
    )
    generate_lookup_table(
        index=2,
        table_c=32,
        table_c_min=10.0,
        table_c_max=30.0,
        table_q=8,
        table_q_min=0.5,
        table_q_max=1.0,
    )
    generate_lookup_table(
        index=3,
        table_c=32,
        table_c_min=30.0,
        table_c_max=100.0,
        table_q=8,
        table_q_min=0.5,
        table_q_max=1.0,
    )
    generate_lookup_table(
        index=4,
        table_c=16,
        table_c_min=100.0,
        table_c_max=700.0,
        table_q=8,
        table_q_min=0.5,
        table_q_max=1.0,
    )

    
def generate_lookup_table(index, table_c, table_c_min, table_c_max, table_q, table_q_min, table_q_max):
    print("pub const TABLE_{}_C: usize = {};".format(index, table_c))
    print("pub const TABLE_{}_C_MIN: f32 = {};".format(index, table_c_min))
    print("pub const TABLE_{}_C_MAX: f32 = {};".format(index, table_c_max))
    print("pub const TABLE_{}_Q: usize = {};".format(index, table_q))
    print("pub const TABLE_{}_Q_MIN: f32 = {};".format(index, table_q_min))
    print("pub const TABLE_{}_Q_MAX: f32 = {};".format(index, table_q_max))
    print("pub static TABLE_{}: [[f32; TABLE_{}_C]; TABLE_{}_Q] = [".format(index, index, index))

    for q in np.linspace(table_q_min, table_q_max, table_q):
        print("[", end="")

        input_configs = []
        for c in np.linspace(table_c_min, table_c_max, table_c):
            config = {
                "c": c,
                "q": q,
            }
            input_configs.append(config)

        # NOTE: Parallelization is done on cutoff since it is expected to be typically
        # the longest list
        configs = []
        with concurrent.futures.ProcessPoolExecutor() as executor:
            # NOTE: Set a lower frequency to get better resolution on zero crossing
            for config in executor.map(partial(set_relative_delay, frequency=0.1), input_configs):
                configs.append(config)

        for c in configs:
            print(c["d"], end=", ")

        print("],")
    print("];")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog=sys.argv[0])
    subparsers = parser.add_subparsers(
        help="sub-command help", required=True, dest="subparser"
    )
    subparsers.add_parser("sliders", help="Adjust parameters and see the delay")
    subparsers.add_parser(
        "generate",
        help="Generates a dataset of phase delays per range of cutoff frequencies and Q factors",
    )
    subparsers.add_parser(
        "lookup",
        help="Uses pre-generated dataset to create a multi-dimensional array of a (cutoff, q_factor) -> phase_delay lookup table",
    )
    args = parser.parse_args()

    if args.subparser == "sliders":
        cmd_sliders()
    elif args.subparser == "generate":
        cmd_generate()
    elif args.subparser == "lookup":
        cmd_lookup()
