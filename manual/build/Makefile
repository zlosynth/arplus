DIAGRAMS = \
	diagram-schema.pdf

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
