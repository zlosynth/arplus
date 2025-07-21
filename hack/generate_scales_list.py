#!/usr/bin/env python
#
# Do not bother reading/changing this file beyond the HEADER definition.
# Everything else is an AI-generated mess. But it works.

import re
import sys
from itertools import product

HEADER = r"""
\version "2.25.26"

\paper {
  #(set-paper-size "a4landscape")
  indent = 0
  line-width = 270\mm
  system-system-spacing.basic-distance = #8
  markup-system-spacing.basic-distance = #16
}

\markup sans_big = \markup \override #'((font-name . "Latin Modern Sans") (font-size . 4)) \etc
\markup sans_small = \markup \override #'((font-name . "Latin Modern Sans") (font-size . 0)) \etc

\header {
  title = \markup \sans_big "Appendix: Scales"
  subsubtitle = \markup \sans_small "All the available scales divided into groups."
  tagline = ##f
}

\layout {
  \context {
    \Score
    \remove "Bar_check_engraver"
  }
  \context {
    \Staff
    \remove "Time_signature_engraver"
  }
}
"""

INTERVALS = {
    "Q": 1, "Q3": 3, "S": 2, "S3": 6, "T": 4, "T2": 8,
}
NOTE_LETTERS = ['c', 'd', 'e', 'f', 'g', 'a', 'b']
NATURAL_Q = {'c': 0, 'd': 4, 'e': 8, 'f': 10, 'g': 14, 'a': 18, 'b': 22}
ACCIDENTAL_STR = {-4:'eses',-3:'eseh',-2:'es',-1:'eh',0:'',1:'ih',2:'is',3:'isih',4:'isis'}
ALLOWED_OFFSETS = {-2, -1, 0, 1, 2}  # Allow flats, half-flats, naturals, half-sharps, sharps

def extract_annotation(comment):
    if not comment: return ""
    comment = comment.strip()
    if ',' in comment:
        return comment.split(',', 1)[0].strip()
    return comment

def parse_rust_scales_with_names(text):
    group_re = re.compile(r'let\s+(\w+)\s*=\s*initialize_group\(&\[(.*?)\]\);', re.DOTALL)
    scale_re = re.compile(
        r'(?:^\s*//\s*(.*?)\n)?^\s*\(\s*&\[(.*?)\],', re.MULTILINE | re.DOTALL)
    groups = []
    for gm in group_re.finditer(text):
        group_name = gm.group(1)
        group_block = gm.group(2)
        scales = []
        for sm in scale_re.finditer(group_block):
            comment = sm.group(1)
            annotation = extract_annotation(comment)
            steps_str = sm.group(2)
            step_syms = [s.strip() for s in steps_str.split(',') if s.strip()]
            scales.append((annotation, step_syms))
        groups.append((group_name, scales))
    return groups

def step_to_q(step): return INTERVALS.get(step, 0)

def lilypond_note(letter, offset, octave):
    acc = ACCIDENTAL_STR.get(offset, '')
    mark = "'" * (octave - 3)
    return f"{letter}{acc}{mark}"

def spelled_scale_unique_if_possible(intervals, allowed_offsets=ALLOWED_OFFSETS, start_pitch_q=0, start_octave=4):
    """Try to spell a scale with unique letter names for each note position"""
    n = len(intervals) + 1
    scale_pitches = [start_pitch_q]
    for interval in intervals:
        scale_pitches.append(scale_pitches[-1] + interval)

    first_octave = start_octave
    last_octave = start_octave if n <= 6 else start_octave + 1

    n_inner = n - 2
    if n_inner <= 0:
        # Handle scales with 2 or fewer notes
        first_pitch = scale_pitches[0]
        first_offset = first_pitch - NATURAL_Q['c']
        if first_offset not in allowed_offsets:
            return None
        last_pitch = scale_pitches[-1]
        last_offset = last_pitch - NATURAL_Q['c'] - (last_octave - start_octave)*24
        if last_offset not in allowed_offsets:
            return None
        return [('c', first_offset, first_octave), ('c', last_offset, last_octave)]

    # Special case: For 7-note scales, try the natural diatonic sequence first
    if n == 8:  # 7 inner notes + 2 C's = 8 total
        diatonic_letters = ['d', 'e', 'f', 'g', 'a', 'b']
        natural_spelling = []
        valid = True
        
        for i, letter in enumerate(diatonic_letters):
            pitch = scale_pitches[i+1]
            # Try octaves around the starting octave
            for octave in range(start_octave-1, start_octave+2):
                natural_q = NATURAL_Q[letter] + (octave - start_octave)*24
                offset = pitch - natural_q
                if offset in allowed_offsets:
                    natural_spelling.append((letter, offset, octave))
                    break
            else:
                valid = False
                break
        
        if valid:
            # Check if the first and last C can be spelled
            first_pitch = scale_pitches[0]
            first_offset = first_pitch - NATURAL_Q['c']
            last_pitch = scale_pitches[-1]
            last_offset = last_pitch - NATURAL_Q['c'] - (last_octave - start_octave)*24
            
            if first_offset in allowed_offsets and last_offset in allowed_offsets:
                spelling = [('c', first_offset, first_octave)]
                spelling.extend(natural_spelling)
                spelling.append(('c', last_offset, last_octave))
                
                # Check for consecutive same letters with different accidentals
                if not has_consecutive_same_letter_different_accidental(spelling):
                    return spelling

    # Build possible choices for each inner note, prioritizing flats over sharps
    possible_inner = []
    PREFERRED_OFFSETS = [-2, -1, 0, 1, 2]  # flats first, then naturals, then sharps

    for i in range(n_inner):
        pitch = scale_pitches[i+1]
        choices = []
        for offset in PREFERRED_OFFSETS:  # Try offsets in preference order
            if offset not in allowed_offsets:
                continue
            for letter in NOTE_LETTERS:
                if letter == 'c':  # Skip C for inner notes
                    continue
                for octave in range(start_octave-1, start_octave+2):
                    natural_q = NATURAL_Q[letter] + (octave - start_octave)*24
                    if natural_q + offset == pitch:
                        choices.append((letter, offset, octave))
        possible_inner.append(choices)

    # Try to find a combination where no letter+octave is repeated
    # and no consecutive notes have same letter with different accidentals
    for combo in product(*possible_inner):
        staff_positions = set()
        ok = True
        for note in combo:
            staff = (note[0], note[2])  # letter + octave
            if staff in staff_positions:
                ok = False
                break
            staff_positions.add(staff)
        
        if ok:
            # Check if we can spell the first and last C notes
            first_pitch = scale_pitches[0]
            first_offset = first_pitch - NATURAL_Q['c']
            if first_offset not in allowed_offsets:
                continue
            
            last_pitch = scale_pitches[-1]
            last_offset = last_pitch - NATURAL_Q['c'] - (last_octave - start_octave)*24
            if last_offset not in allowed_offsets:
                continue
            
            # Build the complete spelling
            spelling = [('c', first_offset, first_octave)]
            spelling.extend(combo)
            spelling.append(('c', last_offset, last_octave))
            
            # Check for consecutive same letters with different accidentals
            if not has_consecutive_same_letter_different_accidental(spelling):
                return spelling
    
    return None

def has_consecutive_same_letter_different_accidental(spelling):
    """Check if a spelling has consecutive notes with same letter but different accidentals"""
    for i in range(len(spelling) - 1):
        letter1, offset1, octave1 = spelling[i]
        letter2, offset2, octave2 = spelling[i + 1]
        
        # Same letter in same octave with different accidentals
        if letter1 == letter2 and octave1 == octave2 and offset1 != offset2:
            return True
    return False



def spelled_scale_general(intervals, start_pitch_q=0, start_octave=4):
    """Fallback spelling that doesn't guarantee uniqueness but tries to be reasonable"""
    scale_pitches = [start_pitch_q]
    for interval in intervals: 
        scale_pitches.append(scale_pitches[-1] + interval)
    
    spelled = []
    prev_letter_index = NOTE_LETTERS.index('c')
    prev_octave = start_octave
    
    for i, pitch_q in enumerate(scale_pitches):
        if i == 0: 
            spelled.append(('c', 0, start_octave))
            continue
        
        if i == len(scale_pitches) - 1:
            # Last note should be C
            for octave in range(prev_octave, prev_octave + 3):
                natural_q = NATURAL_Q['c'] + (octave - start_octave) * 24
                offset = pitch_q - natural_q
                if offset in ALLOWED_OFFSETS:
                    spelled.append(('c', offset, octave))
                    break
            else:
                spelled.append(('c', 0, prev_octave + 1))
            continue
        
        # Find best spelling for inner note
        candidates = []
        
        # Get previous note for consecutive same-letter check
        prev_letter, prev_offset, prev_oct = spelled[-1] if spelled else ('c', 0, start_octave)
        
        # Try all possible letter/octave/offset combinations
        for octave in range(prev_octave, prev_octave+3):
            for letter in NOTE_LETTERS:
                if letter == 'c':  # Skip C for inner notes
                    continue
                natural_q = NATURAL_Q[letter] + (octave-start_octave)*24
                offset = pitch_q - natural_q
                if offset not in ALLOWED_OFFSETS:
                    continue
                
                # Calculate score
                letter_index = NOTE_LETTERS.index(letter)
                
                # Check if this would create consecutive same letter with different accidental
                is_consecutive_same_letter = (letter == prev_letter and octave == prev_oct and offset != prev_offset)
                
                # Base preference: smaller accidentals, slight preference for flats over sharps, forward letter progression
                score = abs(offset) + (0.1 if offset > 0 else 0)
                if letter_index < prev_letter_index:
                    score += 0.5  # Slight penalty for going backwards in alphabet
                
                # BUT: strongly prioritize avoiding consecutive same letters - this overrides flat preference
                priority = 0 if not is_consecutive_same_letter else 1
                
                candidates.append((priority, score, letter, offset, octave, letter_index))
        
        # Sort candidates: first by priority (0 = no consecutive same letter), then by score
        candidates.sort(key=lambda x: (x[0], x[1]))
        
        if candidates:
            _, _, letter, offset, octave, letter_index = candidates[0]
            spelled.append((letter, offset, octave))
            prev_letter_index = letter_index
            prev_octave = octave
        else:
            spelled.append((NOTE_LETTERS[prev_letter_index], 0, prev_octave))
    
    return spelled

def fix_consecutive_same_letters(spelling, start_octave=4):
    """Post-process a spelling to fix consecutive same letters with different accidentals"""
    if not spelling or len(spelling) < 2:
        return spelling
    
    # Make a copy to avoid modifying the original
    fixed = list(spelling)
    
    # Keep trying to fix issues until no more are found
    changed = True
    while changed:
        changed = False
        for i in range(len(fixed) - 1):
            # Skip first and last notes (should always be C)
            if i == 0 or i == len(fixed) - 2:
                continue
                
            curr_letter, curr_offset, curr_octave = fixed[i]
            next_letter, next_offset, next_octave = fixed[i + 1]
            
            # Check if we have consecutive same letters with different accidentals
            if (curr_letter == next_letter and curr_octave == next_octave and 
                curr_offset != next_offset):
                
                # Try to find an enharmonic equivalent for the current note
                curr_pitch = NATURAL_Q[curr_letter] + (curr_octave - start_octave)*24 + curr_offset
                
                # Look for an alternative spelling that doesn't use the same letter
                for alt_letter in NOTE_LETTERS:
                    if alt_letter == curr_letter:  # Skip same letter, but allow C for enharmonic equivalents
                        continue
                    for alt_octave in range(curr_octave-1, curr_octave+2):
                        natural_q = NATURAL_Q[alt_letter] + (alt_octave - start_octave)*24
                        alt_offset = curr_pitch - natural_q
                        if alt_offset in ALLOWED_OFFSETS:
                            # Found a valid enharmonic equivalent
                            fixed[i] = (alt_letter, alt_offset, alt_octave)
                            changed = True
                            break
                    if changed:
                        break
                if changed:
                    break  # Start over from the beginning after making a change
    
    return fixed

def make_scale_notes(step_syms, group_name):
    intervals = [step_to_q(s) for s in step_syms]
    group_name_l = group_name.lower()
    
    # Special handling for quarter-tone scales in the "full" group
    if group_name_l == "full" and len(step_syms) == 24 and all(s == "Q" for s in step_syms):
        # For the quarter-tone scale, generate each quarter-tone step naturally
        # starting from C without forcing the last note to be C
        return make_quarter_tone_scale_notes(intervals)
    
    # Always try unique spelling first for any other scale
    spelling = spelled_scale_unique_if_possible(intervals)
    if spelling is None:
        spelling = spelled_scale_general(intervals)
    
    # Post-process to fix consecutive same letters, especially for Melakarta scales
    if group_name_l == "melakarta":
        spelling = fix_consecutive_same_letters(spelling)
    
    return [lilypond_note(*n) for n in spelling]

def make_quarter_tone_scale_notes(intervals, start_octave=4):
    """Special handling for quarter-tone scales to avoid duplicate C notes"""
    scale_pitches = [0]  # Start at C
    for interval in intervals:
        scale_pitches.append(scale_pitches[-1] + interval)
    
    spelled = []
    current_letter_index = 0  # Start with C
    current_octave = start_octave
    
    for i, pitch_q in enumerate(scale_pitches):
        if i == 0:
            # First note is always C
            spelled.append(('c', 0, start_octave))
            continue
        
        # For subsequent notes, spell them naturally
        # Find the best letter/accidental combination for this pitch
        candidates = []
        
        for letter in NOTE_LETTERS:
            for octave in range(current_octave, current_octave + 2):
                natural_q = NATURAL_Q[letter] + (octave - start_octave) * 24
                offset = pitch_q - natural_q
                if offset in ALLOWED_OFFSETS:
                    # Calculate score favoring smaller accidentals and forward letter progression
                    letter_index = NOTE_LETTERS.index(letter)
                    
                    # Score based on accidental size and letter progression
                    score = abs(offset) + (0.1 if offset > 0 else 0)  # Slight preference for flats
                    
                    # Prefer moving forward in letter sequence
                    if letter_index >= current_letter_index or (current_letter_index == 6 and letter_index == 0):
                        score += 0.0  # Normal progression
                    else:
                        score += 0.5  # Penalty for going backwards
                    
                    candidates.append((score, letter, offset, octave, letter_index))
        
        if candidates:
            candidates.sort()
            _, letter, offset, octave, letter_index = candidates[0]
            spelled.append((letter, offset, octave))
            current_letter_index = letter_index
            current_octave = octave
        else:
            # Fallback to natural note if no valid accidental found
            spelled.append((NOTE_LETTERS[current_letter_index], 0, current_octave))
    
    return [lilypond_note(*n) for n in spelled]

def lilypond_staff_block(group_name, scales):
    lines = [f"% {group_name}", r"\score {", r"  \new Staff {"]
    
    # Add section label with capitalized group name
    capitalized_name = group_name.capitalize()
    lines.append(f'    \\sectionLabel "{capitalized_name}"')
    
    for i, (annotation, scale) in enumerate(scales):
        notes = make_scale_notes(scale, group_name)
        n_notes = len(notes)
        lines.append(f'    \\time {n_notes}/4')
        lily_note_strs = []
        for idx, n in enumerate(notes):
            if idx == 0 and annotation:
                safe_anno = annotation.replace('"', '\\"')
                lily_note_strs.append(f'{n}4^"{safe_anno}"')
            else:
                lily_note_strs.append(f'{n}4')
        lines.append(f"    {' '.join(lily_note_strs)} |")
        
        # Add line break after 4th bar for Melakarta scales
        if group_name.lower() == "melakarta" and i == 3:  # After the 4th scale (index 3)
            lines.append(r"    \break")
    
    lines.append(r'    \bar "||"')
    lines.append(r"  }")
    lines.append(r"}")
    return "\n".join(lines)

def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} input.rs"); sys.exit(1)
    with open(sys.argv[1]) as f: text = f.read()
    groups = parse_rust_scales_with_names(text)
    print(HEADER.strip())
    for group_name, scales in groups:
        print(lilypond_staff_block(group_name, scales))
        print()

if __name__ == '__main__':
    main()
