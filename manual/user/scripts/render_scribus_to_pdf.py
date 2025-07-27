# Generate PDF from the Scribus project
# usage:
# scribus -g -ns -py render_scribus_to_pdf.py Manual.sla Manual-0.pdf

import os
import sys

scribus.openDoc(sys.argv[1])
pdf = scribus.PDFfile()
pdf.file = sys.argv[2]
pdf.save()
