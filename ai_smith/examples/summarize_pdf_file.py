"""
Summarize PDF File

Usage:
    summarize_pdf_file.py <pdf_file>
    summarize_pdf_file.py (-h | --help)

Options:
    -h --help     Show this help message and exit.

Arguments:
    pdf_file    Path to the PDF file to summarize
"""
import queue
import asyncio
from ai_smith import *
import os, sys
import openparse
from docopt import docopt

SUMMARY_FSM_CONFIG = """
states = [
"StandBy",
"GenerateSummary", 
"Finish"]

transitions = [
["StandBy",
"GenerateSummary"],
["GenerateSummary",
"Finish"]]

initial_state = "StandBy"
system_prompt = ""
fsm_prompt = ""
summary_prompt = ""

[state_prompts.StandBy]
system = ""

[state_prompts.GenerateSummary]
system = \"\"\"
Summarize the following research paper using the structured format below. Ensure the summary is clear, concise, and captures the core insights of the paper."

(1) Summary:
Provide a brief overview of the paper, including its main objective, problem statement, and the proposed approach or solution.

(2) Key Contributions:
List the major contributions of the paper. Focus on novel methodologies, frameworks, models, or findings introduced by the authors.

(3) Key Findings:
Summarize the experimental results and insights. Highlight how the proposed approach performs compared to existing methods, including accuracy improvements or any significant trends.

(4) Conclusion:
Summarize the overall impact of the paper, its implications, and potential future directions suggested by the authors.
\"\"\"

[state_config.StandBy]
# don't make chat request but making the fsm transition request
disable_llm_request = true

[state_config.GenerateSummary]
save_to_summary = true

[state_config.Finish]
# don't make chat request but making the fsm transition request
disable_llm_request = true
"""

async def run(pdf_file):
    parser = openparse.DocumentParser()
    parsed_doc = parser.parse(pdf_file)
    parsed_dict = parsed_doc.model_dump()
    text = []
    for node in parsed_dict["nodes"]:
        text.append(node["text"])
    text = " ".join(text)
    print("input text size:", len(text))

    agent_setting = AgentSettings(
            model="gpt-4o-mini",
            api_key=os.getenv("OPENAI_API_KEY"),
            total_state_transition_limit=10
        )

    # Create command and service queues
    command_queue = queue.Queue()
    service_queue = queue.Queue()
    agent = Agent(SUMMARY_FSM_CONFIG, agent_setting)

    agent.agent_message_service(
        command_queue,
        service_queue,
        temperature=0.7
    )

    # Add a test command to the queue
    command_queue.put_nowait(("message", text))
    
    while True:
        try:
            (state, tag, msg) = service_queue.get()
            if tag == "state":
                print()
                print()
                print("--- current state: ", state)
                print()
            elif tag == "token":
                print(msg, end="")
            elif tag == "exec_output":
                print()
                print(msg)
                print()
        except queue.Empty:
            break
        if tag == "message_processed":
            break

    command_queue.put_nowait(("terminate", ""))
    agent.abort_agent_message_service()

def main():
    args = docopt(__doc__)
    pdf_file = args['<pdf_file>']
    
    if not os.path.exists(pdf_file):
        print(f"Error: PDF file '{pdf_file}' does not exist")
        sys.exit(1)
        
    if not pdf_file.lower().endswith('.pdf'):
        print(f"Error: File '{pdf_file}' is not a PDF file")
        sys.exit(1)
        
    asyncio.run(run(pdf_file))

if __name__ == '__main__':
    main()
