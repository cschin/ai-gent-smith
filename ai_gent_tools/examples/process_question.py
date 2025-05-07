import os, subprocess
f = open("questions")
for i, r in enumerate(f):
    r = r.strip().replace('"', '\\"')
    out_file = open(f"q_and_a_{i+1:02d}.log", "w")
    print(f"Question: {r}", file=out_file)
    cmd = f"""
cargo run --release --bin fsm_agent -- --config-file  rag.toml --input "{r}" """
    print(file=out_file)
    print("Start Ai-smith", file=out_file)
    print(file=out_file)
    process = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    stdout, stderr = process.communicate()
    print(f"{stdout}", file=out_file)
