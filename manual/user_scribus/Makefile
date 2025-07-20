DIAGRAMS = \
	panel.pdf \
	arp-symbol.pdf \
	signal-flow.pdf \
	block-root-note.pdf

pdf-images: $(DIAGRAMS)
	touch $@

%.pdf: %.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex

.PHONY: chords
chords:
	../../hack/generate_chords_list.py ../../control/src/chords.rs > chords.ly
	lilypond chords.ly

.PHONY: scales
scales:
	../../hack/generate_scales_list.py ../../control/src/scales/mod.rs > scales.ly
	lilypond scales.ly
