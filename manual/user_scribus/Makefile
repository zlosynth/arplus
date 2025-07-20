DIAGRAMS = \
	panel.pdf \
	arp-symbol.pdf \
	signal-flow.pdf

pdf-images: $(DIAGRAMS)
	touch $@

%.pdf: %.svg
	inkscape --export-text-to-path -D $< -o $@ --export-latex
