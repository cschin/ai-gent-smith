import queue
import asyncio
from ai_smith import *
import os

SAMPLE_FSM_CONFIG = """
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


def agent_settings():
    return AgentSettings(
        model="gpt-3.5-turbo",
        api_key="test-api-key",
        total_state_transition_limit=10
    )


async def run(): 
    agent_setting = AgentSettings(
            model="gpt-3.5-turbo",
            api_key=os.getenv("OPENAI_API_KEY"),
            total_state_transition_limit=10
        )

    # Create command and service queues
    command_queue = queue.Queue()
    service_queue = queue.Queue()
    agent = Agent(SAMPLE_FSM_CONFIG, agent_setting)

    agent.agent_message_service(
        command_queue,
        service_queue,
        temperature=0.7
    )


    # Add a test command to the queue
    command_queue.put_nowait(("message", "test_payload1"))
    command_queue.put_nowait(("terminate", ""))
    
    while True:
        try:
            out = service_queue.get()
            print(out)
        except queue.Empty:
            break
        if out[1] == "message_processed":
            break
    print("Done")

    #agent.stop_agent_message_service()

asyncio.run(run())
