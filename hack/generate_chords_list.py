#!/usr/bin/env python
#
# Do not bother reading/changing this file beyond the HEADER definition.
# Everything else is an AI-generated mess. But it works.

import re
import sys

HEADER = r"""
\version "2.25.26"

\paper {
  #(set-paper-size "a4landscape")
  indent = 0
  line-width = 270\mm
  system-system-spacing.basic-distance = #6
  markup-system-spacing.basic-distance = #16
}

\header {
  title = "List of chords"
  subsubtitle = "All the available chords grouped by size. The actual notes will differ based on the selected scale and tonic."
  tagline = ##f
}

\layout {
  \context {
    \Staff
    \remove "Time_signature_engraver"
  }
}
"""

def degree_to_note(degree):
    notes = ['c', 'd', 'e', 'f', 'g', 'a', 'b']
    octave = 4 + (degree // 7)
    note_name = notes[degree % 7]
    mark = "'" * (octave - 3)
    return f"{note_name}{mark}"

def parse_groups(rust_code):
    group_pattern = re.compile(
        r'let (size_\d) = initialize_group\(&\[(.*?)\]\);',
        re.DOTALL
    )
    groups = []
    for m in group_pattern.finditer(rust_code):
        group_name = m.group(1)
        body = m.group(2)
        chords = []
        for chord_line in re.findall(r'&\[(.*?)\],', body, re.DOTALL):
            chord_exprs = [x.strip() for x in chord_line.split(',')]
            chord_degrees = []
            for e in chord_exprs:
                if e == '':
                    continue
                try:
                    chord_degrees.append(eval(e))
                except Exception:
                    chord_degrees.append(0)
            chords.append(chord_degrees)
        groups.append((group_name, chords))
    return groups

def lilypond_score_block(group_name, group_idx, chords):
    size = group_idx + 1
    lines = []
    lines.append(f"% {group_name}")
    lines.append(r"\score {")
    lines.append(r"  \new Staff {")
    lines.append(f"    \\sectionLabel \"Size: {size}\"")
    for i, chord in enumerate(chords):
        # Double space between notes
        notes = "  ".join(degree_to_note(d) for d in chord)
        lines.append(f"    <{notes}>1")
    lines.append(r'    \bar "||"')
    lines.append(r"  }")
    lines.append(r"}")
    return "\n".join(lines)

def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} input.rs")
        sys.exit(1)
    input_file = sys.argv[1]
    with open(input_file) as f:
        rust_code = f.read()

    groups = parse_groups(rust_code)

    print(HEADER.strip() + "\n")
    for idx, (group_name, chords) in enumerate(groups):
        print(lilypond_score_block(group_name, idx, chords))
        print()

if __name__ == '__main__':
    main()
