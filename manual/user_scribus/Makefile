DIAGRAMS = \
	panel.pdf \
	arp-symbol.pdf \
	signal-flow.pdf \
	block-root-note.pdf

pdf-images: $(DIAGRAMS)
	touch $@

%.pdf: %.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex
