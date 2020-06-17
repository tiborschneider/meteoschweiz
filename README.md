# MeteoSchweiz
This program fetches forecast data from [meteoschweiz.admin.ch](https://www.meteoschweiz.admin.ch), generates a pdf using TikZ, and displays the result.

## Dependencies
* wget (for downloading the svg icons)
* inkscape (for converting the svg icons to pdf)
* pdflatex (for rendering the image, requires `tikz`, `pgfplot`, `xcolor` and `graphicx` per default)
* zathura (or any other program to display pdf files)

## Usage
After the first start, `meteoschweiz` will setup the config directory, create the documented sample configuration file and both templates. Then, it downloads all necessary icons from [meteoschweiz.admin.ch](https://www.meteoschweiz.admin.ch/etc/designs/meteoswiss/assets/images/icons/meteo/weather-symbols/1.svg) and converts them to pdf.

All rendered images and the parsed data is cached in the cache folder for quicker access. If a new forecast is available online, then it is fetched and rendered again.

## Templates
There are two different templates. You can modify them to your liking. If the template does not exists, the default template will be created again. So if you have messed up the template, just rename or delete it, and the default template will be restored. The template is written in LaTeX, and annotated with [tera](https://tera.netlify.app/docs/).
