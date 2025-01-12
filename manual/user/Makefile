DIAGRAMS = \
	diagram-panel.pdf \
	diagram-flow.pdf \
	diagram-tone-pot.pdf \
	diagram-tone-pot-led.pdf \
	diagram-tone-pot-led-example.pdf \
	diagram-tone-cv.pdf \
	diagram-tone-cv-led.pdf \
	diagram-tone-cv-led-example.pdf \
	diagram-chord-size.pdf \
	diagram-chord-size-led.pdf \
	diagram-chord-size-led-example.pdf \
	diagram-chord-pot.pdf \
	diagram-chord-cv.pdf \
	diagram-chord-led.pdf \
	diagram-chord-led-example.pdf \
	diagram-arp-select.pdf \
	diagram-arp-query.pdf \
	diagram-arp-progression-click.pdf \
	diagram-arp-progression-cv.pdf \
	diagram-arp-progression-explanation.pdf \
	diagram-arp-rsnx-click.pdf \
	diagram-arp-rsnx-cv.pdf \
	diagram-arp-rsnx-explanation.pdf \
	diagram-group-select.pdf \
	diagram-group-query.pdf \
	diagram-scale-select.pdf \
	diagram-scale-query.pdf \
	diagram-tonic.pdf \
	diagram-tonic-led.pdf \
	diagram-tonic-led-example.pdf \
	diagram-tonic-query.pdf \
	diagram-quantized.pdf \
	diagram-karplus-strong.pdf \
	diagram-audio-output.pdf \
	diagram-karplus-strong-noise-explained.pdf \
	diagram-karplus-strong-filter-explained.pdf \
	diagram-pluck-pot.pdf \
	diagram-contour-pot.pdf \
	diagram-contour-cv.pdf \
	diagram-cutoff-pot.pdf \
	diagram-cutoff-cv.pdf \
	diagram-resonance-pot.pdf \
	diagram-resonance-cv.pdf \
	diagram-mapping-led.pdf

pdf: pdf-images manual.tex
	pdflatex manual.tex
	pdflatex manual.tex
	touch $@

pdf-images: $(DIAGRAMS)
	touch $@

diagram-%.pdf: diagram-%.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex

.PHONY: clean
clean:
	rm -rf *.pdf *.pdf_tex *.log *.aux .auctex-auto
