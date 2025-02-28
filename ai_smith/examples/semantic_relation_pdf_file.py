"""
Find Semantic Relationship in a PDF file

Usage:
    semantic_relation_pdf_file.py <pdf_file>
    semantic_relation_pdf_file.py (-h | --help)

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
"IdentifyRelations", 
"Finish"]

transitions = [
["StandBy",
"IdentifyRelations"],
["IdentifyRelations",
"Finish"]]

initial_state = "StandBy"
system_prompt = ""
fsm_prompt = ""
summary_prompt = ""

[state_prompts.StandBy]
system = ""

[state_prompts.IdentifyRelations]
system = \"\"\"
Given biological texts, extract meaningful semantic relationships between biological entities in the form of RDF-like triples. Each triple should be structured as (Subject, Predicate, Object), where:

Subject represents a biological entity (e.g., gene, protein, organism, process, disease).
Predicate defines the relationship between the subject and object (e.g., activates, inhibits, is_part_of, causes).
Object represents another biological entity, concept, or attribute.
Example Input:
"The BRCA1 gene regulates DNA repair and mutations in BRCA1 can lead to breast cancer."

Example Output (Triples):

(BRCA1, regulates, DNA repair)
(Mutation_in_BRCA1, causes, Breast Cancer)
Extract and generate biologically relevant triples while ensuring the relationships maintain logical and hierarchical connections within the biological domain.
\"\"\"

[state_config.StandBy]
# don't make chat request but making the fsm transition request
disable_llm_request = true

[state_config.IdentifyRelations]
# save_to_summary = true

[state_config.Finish]
# don't make chat request but making the fsm transition request
disable_llm_request = true
"""

async def run(pdf_file):

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

    parser = openparse.DocumentParser()
    parsed_doc = parser.parse(pdf_file)
    parsed_dict = parsed_doc.model_dump()
    text = []
    count = 0
    for node in parsed_dict["nodes"]:
        text=node["text"]
        print("input text size:", len(text))

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

        command_queue.put_nowait(("clear_message", text))
        if count > 20:
            break

        count += 1

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
