DIAGRAMS = \
	diagram-panel.pdf \
	diagram-flow.pdf \
	diagram-tone-pot.pdf \
	diagram-tone-pot-led.pdf \
	diagram-tone-pot-led-example.pdf \
	diagram-tone-cv.pdf \
	diagram-tone-cv-led.pdf \
	diagram-tone-cv-led-example.pdf \
	diagram-chord-pot.pdf \
	diagram-chord-cv.pdf \
	diagram-chord-led.pdf \
	diagram-chord-led-example.pdf \
	diagram-chord-size.pdf \
	diagram-chord-size-led.pdf \
	diagram-chord-size-led-example.pdf \
	diagram-mapping-led.pdf \
	diagram-arp.pdf \
	diagram-arp-query.pdf \
	diagram-arp-select.pdf \
	diagram-arp-progression-click.pdf \
	diagram-arp-progression-cv.pdf \
	diagram-arp-progression-explanation.pdf \
	diagram-arp-rsnx-click.pdf \
	diagram-arp-rsnx-cv.pdf \
	diagram-arp-rsnx-explanation.pdf

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
