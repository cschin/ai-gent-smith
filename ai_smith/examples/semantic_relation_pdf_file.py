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
Analyze the provided domain-specific text or dataset and extract all semantic relationships between key entities. Each relationship should be expressed as a triple consisting of Subject, Predicate, and Object. Your output must be structured in JSON format.

Instructions:

- Entity Extraction:

    -- Identify the primary entities (concepts, objects, or classes) in the text.
    -- These will serve as the subjects and objects in your triples.

- Relationship Identification:
    -- Determine the relationships (predicates) that connect these entities.
    -- Use meaningful predicates that reflect the ontology’s semantics (e.g., isA, hasPart, treats, causes, etc.).

- Ontology Standards:
    -- Where applicable, utilize standardized relationships from common ontology frameworks (e.g., RDF Schema or OWL).

- Disambiguation:
    -- Provide context for entities with multiple meanings to ensure clear semantic interpretation.

- JSON Output Format:

    -- Format your output as a JSON object with a key "triples".
    -- The value should be an array of objects, where each object represents a triple with three keys: "subject", "predicate", and "object".

Example Output:
{
  "triples": [
    {
      "subject": "Diabetes",
      "predicate": "isA",
      "object": "Disease"
    },
    {
      "subject": "Insulin",
      "predicate": "treats",
      "object": "Diabetes"
    },
    {
      "subject": "Heart Disease",
      "predicate": "riskFactor",
      "object": "Smoking"
    },
    {
      "subject": "Car",
      "predicate": "hasPart",
      "object": "Engine"
    }
  ]
}

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
