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
from csv import DictWriter

import numpy as np
import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.widgets import Slider
from scipy.optimize import curve_fit


DELAY_DATASET = "delay_dataset.csv"

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


def set_relative_delay(config):
    SAMPLE_RATE = 48_000
    FREQUENCY = 50.0

    time = 3 / FREQUENCY
    
    input = generate_sine(FREQUENCY, time, SAMPLE_RATE)
    filtered = low_pass_filter(input, config["c"] * FREQUENCY, config["q"], SAMPLE_RATE)

    input_zero_crossings = np.where(np.diff(np.sign(input)))[0]
    filtered_zero_crossings = np.where(np.diff(np.sign(filtered)))[0]
    fifth_input_zero_crossing_in_seconds = input_zero_crossings[4] / SAMPLE_RATE;
    fifth_filtered_zero_crossing_in_seconds = filtered_zero_crossings[4] / SAMPLE_RATE;
    delay_in_seconds = fifth_filtered_zero_crossing_in_seconds - fifth_input_zero_crossing_in_seconds;
    interval = 1.0 / FREQUENCY;
    config["d"] = delay_in_seconds / interval;

    return config


def cmd_delay_generate():
    C = 100
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


def cmd_delay_fitting():
    try:
        data_frame = pd.read_csv(DELAY_DATASET)
    except FileNotFoundError:
        exit("Dataset not found, generate it first")

    d_data = data_frame["d"].values
    c_data = data_frame["c"].values
    q_data = data_frame["q"].values

    functions = (
        func_j_pow2,  # rmse=0.00198
        func_f_pow2,  # rmse=0.00391
        func_d_exp,  # rmse=0.00630
        func_g_pow2,  # rmse=0.00740
        func_e_pow2,  # rmse=0.00834
        func_c_pow2,  # rmse=0.00839
        func_h_pow2,  # rmse=0.00907
        func_b_simple,  # rmse=0.01638
        func_a_linear,  # rmse=0.01674
        func_k_exp,  # rmse=1.0
        func_i_pow2,  # rmse=1.0
    )

    input_configs = [(func, c_data, q_data, d_data) for func in functions]
    test_fitting_functions(input_configs)


def func_a_linear(data, a1, a2, b):
    c, q = data[0], data[1]
    return a1 * c + a2 * q + b


def func_b_simple(data, a1, a2, a4, b):
    c, q = data[0], data[1]
    return a1 * c + a2 * q + a4 * c * q + b


def func_c_pow2(data, a1, a2, a4, a7, a8, b):
    c, q = data[0], data[1]
    return (
        a1 * c
        + a2 * q
        + a4 * c * q
        + a7 * c**2
        + a8 * q**2
        + b
    )


def func_d_exp(data, a1, a2, a4, a7, a8, a9, a10, b):
    c, q = data[0], data[1]
    return (
        a1 * c
        + a2 * q
        + a4 * c * q
        + a7 * c**a8
        + a9 * q**a10
        + b
    )


def func_e_pow2(data, a1, a2, a3, a4, a5, a6, b):
    c, q = data[0], data[1]
    return (a1 + a2 * c + a3 * c**2) / (a4 + a5 * q + a6 * q**2) + b


def func_f_pow2(data, a1, a2, a3, a4, a5, a6, b):
    c, q = data[0], data[1]
    return (a1 + a2 * c + a3 * q**2) / (a4 + a5 * q + a6 * c**2) + b


def func_g_pow2(data, a1, a2, a4, a5, a6, a7, a9, b):
    c, q = data[0], data[1]
    return (a1 + a2 * c) / (
        (a4 + a5 * q + a6 * c**2) * (a7 + a9 * q**2)
    ) + b


def func_h_pow2(data, a1, a2, a3, a4, a5, a6, a7, b):
    c, q = data[0], data[1]
    return ((a1 + a2 * c + a3 * c**2) * (a4 + a5 * q + a6 * q**2)) / a7 + b


def func_i_pow2(data, a1, a2, a3, a4, a5, a6, a7, b):
    c, q = data[0], data[1]
    return ((a1 + a2 * c + a3 * q**2) * (a4 + a5 * q + a6 * c**2)) / a7 + b


def func_j_pow2(data, a1, a2, a4, a5, a6, a7, a9, b):
    c, q = data[0], data[1]
    return ((a1 + a2 * c) * (a4 + a5 * q + a6 * q**2)) / (a7 + a9 * c**2) + b


def func_k_exp(data, a1, a2, a3, a4, a5, a6, a7, b):
    c, q = data[0], data[1]
    return (a1 + a2 * c**a3) / ((a4 + a5 * q**a6) * a7) + b



def func_q_pow3(
    data,
    a1,
    a2,
    a3,
    a4,
    a5,
    a6,
    a7,
    a8,
    a9,
    a10,
    a11,
    a12,
    a13,
    a14,
    a15,
    a16,
    a17,
    a18,
    a19,
    b,
):
    d, s, w = data[0], data[1], data[2]
    return (
        a1 * d
        + a2 * s
        + a3 * w
        + a4 * d**2
        + a5 * s**2
        + a6 * w**2
        + a7 * d**3
        + a8 * s**3
        + a9 * w**3
        + a10 * d * s
        + a11 * d * w
        + a12 * s * w
        + a13 * d**2 * s
        + a14 * d * s**2
        + a15 * d**2 * w
        + a16 * d * w**2
        + a17 * s**2 * w
        + a18 * s * w**2
        + a19 * d * s * w
        + b
    )


def test_fitting_functions(input_configs):
    print("Testing fitting functions:")
    configs = []
    with concurrent.futures.ProcessPoolExecutor() as executor:
        for i, config in enumerate(
            executor.map(measure_fitting_accuracy, input_configs)
        ):
            configs.append(config)
            print("Processed: {}/{}".format(i + 1, len(input_configs)))

    configs = sorted(configs, key=lambda x: x["rmse"])

    print("Approximations ordered by accuracy:")
    for (i, config) in enumerate(configs):
        print(
            "{}. {}(rmse={}, rs={})".format(
                i + 1, config["func"], config["rmse"], config["rs"]
            )
        )

    while True:
        selected = input("Show parameters (empty to exit): ")
        if selected == "":
            return
        try:
            parameters = configs[int(selected) - 1]["parameters"]
        except:
            print("Invalid index")
            continue
        print_parameters(parameters)


def print_parameters(parameters):
    for (i, a) in enumerate(parameters[: len(parameters) - 1]):
        print("a{} = {}".format(i + 1, a))
    print("b = {}".format(parameters[-1]))


def measure_fitting_accuracy(config):
    f = config[0]

    try:
        fitted_parameters, _ = curve_fit(f, config[1:-1], config[-1], maxfev=60000)
    except RuntimeError:
        print("Unable to fit")
        return {
            "func": f.__name__,
            "rmse": 1.0,
            "rs": 0.0,
            "parameters": [],
        }

    model_predictions = f(config[1:], *fitted_parameters)
    abs_errors = model_predictions - config[-1]
    squared_errors = np.square(abs_errors)
    mean_squared_errors = np.mean(squared_errors)
    root_mean_squared_errors = np.sqrt(mean_squared_errors)
    r_squared = 1.0 - (np.var(abs_errors) / np.var(config[-1]))
    return {
        "func": f.__name__,
        "rmse": root_mean_squared_errors,
        "rs": r_squared,
        "parameters": fitted_parameters,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog=sys.argv[0])
    subparsers = parser.add_subparsers(
        help="sub-command help", required=True, dest="subparser"
    )
    subparsers.add_parser("sliders", help="Adjust parameters and see the delay")
    subparsers.add_parser("delay_generate", help="TODO")
    subparsers.add_parser("delay_fitting", help="TODO")
    args = parser.parse_args()

    if args.subparser == "sliders":
        cmd_sliders()
    elif args.subparser == "delay_generate":
        cmd_delay_generate()
    elif args.subparser == "delay_fitting":
        cmd_delay_fitting()
