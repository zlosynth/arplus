DIAGRAMS = \
	diagram-panel.pdf \
	diagram-flow.pdf \
	diagram-tone.pdf \
	diagram-chord.pdf \
	diagram-arp.pdf

.PHONY: pdf
pdf: clean pdf-images
	pdflatex manual.tex
	pdflatex manual.tex

.PHONY: pdf-images
pdf-images: $(DIAGRAMS)
	# inkscape -D schema.svg -o schema.pdf --export-latex

diagram-%.pdf: diagram-%.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex

.PHONY: clean
clean:
	rm -rf *.pdf *.pdf_tex *.log *.aux .auctex-auto
