"""
Q&A about a URL File

Usage:
    url_QA.py <url> 
    url_QA.py (-h | --help)

Options:
    -h --help     Show this help message and exit.

Arguments:
    url    URL
"""
import queue
import asyncio
from ai_smith import *
import os, sys
import openparse
from docopt import docopt
import time
from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.chrome.options import Options
from bs4 import BeautifulSoup
from webdriver_manager.chrome import ChromeDriverManager
import readline 
def setup_readline():
    # Enable tab completion
    readline.parse_and_bind('tab: complete')
    
    # Set up command history file
    histfile = '.url_qa_history'
    try:
        readline.read_history_file(histfile)
        # Set history length to 1000 entries
        readline.set_history_length(1000)
    except FileNotFoundError:
        pass

    return histfile

def cleanup_readline(histfile):
    # Save command history on exit
    readline.write_history_file(histfile)



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

async def run(url):
    # Setup Chrome options
    chrome_options = Options()
    chrome_options.add_argument("--headless=new")  # Use newer headless mode
    chrome_options.add_argument("--disable-gpu")
    chrome_options.add_argument("--no-sandbox")
    chrome_options.add_argument("--disable-blink-features=AutomationControlled")  # Prevent bot detection

    # Set User-Agent to mimic a real browser
    chrome_options.add_argument(
        "user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    )

    # Initialize WebDriver
    service = Service(ChromeDriverManager().install())
    driver = webdriver.Chrome(service=service, options=chrome_options)

    # Open the webpage
    driver.get(url)

    # Wait for the page to fully load
    time.sleep(5)  # Adjust if necessary for heavy JavaScript sites

    # Get fully rendered HTML
    html = driver.page_source

    # Close the browser
    driver.quit()

    # Parse HTML with BeautifulSoup
    soup = BeautifulSoup(html, "html.parser")

    # Extract and clean text
    text = soup.get_text(separator="\n", strip=True)

    print("text len:", len(text))


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
    histfile = setup_readline()
    command_queue.put_nowait(("push_context", text))
    try:
        while True:
            try:
                # Get user input with readline
                print()
                print()
                user_input = input("Enter your question: ")

                # Check if user wants to quit
                if user_input.lower() == 'quit':
                    command_queue.put_nowait(("terminate", ""))
                    break

            except EOFError:
                print("\nReceived EOF. Exiting...")
                break

            except asyncio.CancelledError:
                print("\nReceived cancel signal. Cleaning up...")
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

    finally:
        # Clean up readline history when exiting
        cleanup_readline(histfile)

    command_queue.put_nowait(("terminate", ""))
    agent.abort_agent_message_service()

def main():
    args = docopt(__doc__)
    url = args['<url>']
    
    try:
        asyncio.run(run(url))
    except KeyboardInterrupt:
        print("\nReceived keyboard interrupt. Exiting...")

if __name__ == '__main__':
    main()
