DIAGRAMS = \
	graphics/panel.pdf \
	graphics/arp-symbol.pdf \
	graphics/signal-flow.pdf \
	graphics/block-root-note.pdf \
	graphics/block-chord.pdf \
	graphics/block-arp.pdf

Manual.pdf: main.pdf scales.pdf chords.pdf
	pdfunite $^ $@

main.pdf: main.sla
	scribus -g -ns -py scripts/render_scribus_to_pdf.py $< $@

scales.pdf: scales.ly
	lilypond $<

scales.ly: ../../control/src/scales/mod.rs
	python scripts/generate_scales_list.py $< > $@

chords.pdf: chords.ly
	lilypond $<

chords.ly: ../../control/src/chords.rs
	python scripts/generate_chords_list.py $< > $@

# This is a helper used to convert SVGs to PDFs that would be then imported
# into Scribus.
pdf-images: $(DIAGRAMS)
	touch $@

%.pdf: %.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex

.PHONY: clean
clean:
	rm -rf *.pdf *.ly
