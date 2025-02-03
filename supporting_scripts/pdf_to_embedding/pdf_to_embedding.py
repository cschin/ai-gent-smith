"""PDF to Embedding

Usage:
  pdf_to_embedding.py [-i <dir>] [-o <file>]
  pdf_to_embedding.py -h | --help

Options:
  -h --help             Show this screen.
  -i --input-dir=<dir>  Input directory containing PDF files [default: .]
  -t --title=<file>     An optional file mapping the PDF file name (full path) to display title (each line is separated by a tab)
  -o --output-file=<file>  Output file for JSON results [default: -]

"""
from docopt import docopt
import openparse
import glob
import sys
import requests
import json
import os
import numpy as np
import umap

def parse_title_file(title_file):
    title_mapping = {}
    if title_file:
        try:
            with open(title_file, 'r') as f:
                for line in f:
                    parts = line.strip().split('\t')
                    if len(parts) == 2:
                        pdf_path, display_title = parts
                        title_mapping[pdf_path] = display_title
        except FileNotFoundError:
            print(f"Warning: Title file '{title_file}' not found. Proceeding without custom titles.")
    return title_mapping

def main(args):
    input_dir_path = args['--input-dir']
    output_file = args['--output-file']
    if '--title' in args:
        title_file = args['--title']
        title_mapping = parse_title_file(title_file)
    else:
        title_mapping = {}

    fns = glob.glob(f"{input_dir_path}/*.pdf") + glob.glob(f"{input_dir_path}/*.PDF")

    out_file = sys.stdout if output_file == '-' else open(output_file, "w")

    url = "http://127.0.0.1:8080/api/service/text_to_embedding"
    headers = {
        "Content-Type": "application/json"
    }
    embedding_chunks = []
    embedding_vec =[]
    for fn in fns:
        print(f"processing file {fn}", file=sys.stderr)
        base_filename = os.path.basename(fn)
        title = title_mapping.get(fn, os.path.splitext(base_filename)[0])
        basic_doc_path = fn
        parser = openparse.DocumentParser()
        parsed_basic_doc = parser.parse(basic_doc_path)
        parsed_dict = parsed_basic_doc.model_dump()
        # print(parsed_dict)
        for node in parsed_dict["nodes"]:
            payload = {"text": node["text"]}
            response = requests.post(url, headers=headers, data=json.dumps(payload))
            if response.status_code == 200:
            # Request was successful
                result = response.json()
                for chunk in result["data"]:
                    if len(chunk["text"]) == 0:
                        continue
                    doc_chunk = {"filename": base_filename, 
                                "title": title, 
                                "two_d_embedding": [0.0, 0.0], 
                                "embedding_vec": chunk["embedding_vec"], 
                                "text":chunk["text"],
                                "span":chunk["span"],
                                "bbox":node["bbox"],
                                "node_id":node["node_id"]}
                    embedding_chunks.append(doc_chunk)
                    embedding_vec.append(chunk["embedding_vec"]),
                    #doc_chunk_json = json.dumps(doc_chunk)
                    #print(doc_chunk_json, file=out_file)
                
            else:
                # Request failed
                print("Error:", response.status_code, response.text)
        print(f"finish processing file {fn}", file=sys.stderr)

    print("compute 2d embedding", file=sys.stderr)
    embedding_vec = np.array(embedding_vec)

    reducer = umap.UMAP(n_components=2)
    embedding_2d = reducer.fit_transform(embedding_vec)
    
    x = embedding_2d[:, 0] 
    y = embedding_2d[:, 1]
    minx = np.min(x)
    maxx = np.max(x)
    miny = np.min(y)
    maxy = np.max(y)
    embedding_2d[:, 0] = 2.0 * (x - minx) / (maxx-minx) - 1.0 
    embedding_2d[:, 1] = 2.0 * (y - miny) / (maxy-miny) - 1.0

    print("writing output", file=sys.stderr)
    for (chunk, two_d_vec) in zip(embedding_chunks, embedding_2d):
        chunk["two_d_embedding"] = [float(two_d_vec[0]), float(two_d_vec[1])] 
        print(json.dumps(chunk),  file=out_file)
        # print(chunk["span"], len(chunk["text"]), file=out_file)

    if out_file != sys.stdout:
        out_file.close()

if __name__ == '__main__':
    arguments = docopt(__doc__)
    main(arguments)
