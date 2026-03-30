PHOTOS = \
	p01.jpg \
	p02.jpg \
	p03.jpg \
	p04.jpg \
	p05.jpg \
	p06.jpg

pdf: $(PHOTOS) manual.tex
	pdflatex manual.tex
	pdflatex manual.tex
	touch $@

.PHONY: clean
clean:
	rm -rf *.pdf *.pdf_tex *.log *.aux .auctex-auto pdf
