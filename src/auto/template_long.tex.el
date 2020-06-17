(TeX-add-style-hook
 "template_long.tex"
 (lambda ()
   (TeX-run-style-hooks
    "latex2e"
    "standalone"
    "standalone10"
    "graphicx"
    "tikz"
    "pgfplots"
    "xcolor")
   (LaTeX-add-xcolor-definecolors
    "tempcol"
    "raincol"))
 :latex)

