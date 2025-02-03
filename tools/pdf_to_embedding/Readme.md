# Offline tool scripts for Ai-gent Smith 

## pdf_to_embedding.py

As simple script to generate the embedding data that can be imported to 
Ai-gent Smit

### Dependences

- [openparse](https://github.com/Filimoa/open-parse)
- [umap-learn](https://github.com/lmcinnes/umap)
- [docopt](https://github.com/docopt/docopt)

install the dependences 

```
pip install openparse umap-learn docopt
```

### Usage

After starting the `ai_gent_web` server, you can send http request to 
`http://127.0.0.1:8080/api/service/text_to_embedding` to get embedding
vectors. The `pdf_to_embedding.py` uses the service to get `.jsonl` file
that can be imported to the `ai_gent_web` to create a new asset.

You need to start the `ai_gent_web` before running `pdf_to_embedding.py`.  

```
$ python pdf_to_embedding.py --help
PDF to Embedding

Usage:
  pdf_to_embedding.py [-i <dir>] [-o <file>]
  pdf_to_embedding.py -h | --help

Options:
  -h --help             Show this screen.
  -i --input-dir=<dir>  Input directory containing PDF files [default: .]
  -t --title=<file>     An optional file mapping the PDF file name (full path) to display title (each line is separated by a tab)
  -o --output-file=<file>  Output file for JSON results [default: -]
```

