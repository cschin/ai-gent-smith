"""
Q&A about a PDF File

Usage:
    pdf_QA.py <pdf_file>
    pdf_QA.py (-h | --help)

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
"AnswerQuestion", 
"Finish"]

transitions = [
["StandBy",
"AnswerQuestion"],
["AnswerQuestion",
"Finish"]]

initial_state = "StandBy"
system_prompt = ""
fsm_prompt = ""
summary_prompt = ""

[state_prompts.StandBy]
system = ""

[state_prompts.AnswerQuestion]
system = \"\"\"
You are an excellent scientific research assistant. You are helping 
scientists to examine a research paper.

You will be given a long context from a research paper. Here is the context

BEGIN OF THE CONTEXT

 {{ context }}

END OF THE CONTEXT

Please answer the scientist's question accordingly

\"\"\"

[state_config.StandBy]
# don't make chat request but making the fsm transition request
disable_llm_request = true

[state_config.AnswerQuestion]
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

    command_queue.put_nowait(("push_context", text))
    while True:
        try:
            # Get user input
            print()
            print()
            user_input = input("Enter your question: ")

            # Check if user wants to quit
            if user_input.lower() == 'quit':
                command_queue.put_nowait(("terminate", ""))
                break
            # Add a test command to the queue

        except EOFError:
            print("\nReceived EOF. Exiting...")
            command_queue.put_nowait(("terminate", ""))
            break
        
        except KeyboardInterrupt:
            print("\nReceived keyboard interrupt. Exiting...")
            command_queue.put_nowait(("terminate", ""))
            break

        command_queue.put_nowait(("message", user_input))
        
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
