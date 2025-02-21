from typing import Optional, List, Dict, Tuple
from pydantic import BaseModel
import toml
from agent_config import *


def create_code_agent_config() -> LlmFsmAgentConfig:

    stand_by = (StateBuilder()
                .set_name("StandBy")
                .set_prompts(
                    StatePromptsBuilder()
                    .set_system("")
                    .set_fsm(r"""JUST output a json string {"next_state": "GatherFact"}""")
                    .build())
                .set_config(
                    StateConfigBuilder()
                    .set_disable_llm_request(True)
                    .build())
                .build())
    
    gather_fact = (StateBuilder()
                   .set_name("GatherFact")
                   .set_prompts(
                        StatePromptsBuilder()
                        .set_system(r""" 
Below I will present you a task.

    You will now build a comprehensive preparatory survey of which facts we have at our disposal and which ones we still need.
    To do so, you will have to read the task and identify things that must be discovered in order to successfully complete it.
    Don't make any assumptions. For each item, provide a thorough reasoning. Here is how you will structure this survey:

    ---
    ### 1. Facts given in the task
    List here the specific facts given in the task that could help you (there might be nothing here).

    ### 2. Facts to look up
    List here any facts that we may need to look up.
    Also list where to find each of these, for instance a website, a file... - maybe the task contains some sources that you should re-use here.

    ### 3. Facts to derive
    List here anything that we want to derive from the above by logical reasoning, for instance computation or simulation.

    Keep in mind that "facts" will typically be specific names, dates, values, etc. Your answer should use the below headings:
    ### 1. Facts given in the task
    ### 2. Facts to look up
    ### 3. Facts to derive
    Do not add anything else.
""")
                        .set_fsm(r"""JUST output a json string {"next_state": "Planning"}""")
                        .build())
                     .set_config(
                        StateConfigBuilder()
                        .set_save_to_context(True)
                        .set_save_to("facts")
                        .build())
                     .build())

    update_fact = (StateBuilder()
                   .set_name("UpdateFact")
                   .set_prompts(
                        StatePromptsBuilder()
                        .set_system(r"""
    You are trying to perform this task: <TASK> {{ task }} </TASK>
   
    Earlier we've built an update list of facts.
    And this it the previous fact you know about: <FACTS> {{ facts }} </FACTS>
   
    After performing the previous steps, you have learned some new facts and invalidated some false ones.
      
    But since in your previous steps you may have learned useful new facts or invalidated some false ones.
    Here is new additional information you have learned so far:
    
        {{output_for_evaluation}}

    Please update your list of facts based on the previous history, and provide these headings:
    ### 1. Facts given in the task
    ### 2. Facts that we have learned
    ### 3. Facts still to look up
    ### 4. Facts still to derive
                                    """)
                        .set_fsm(r"""JUST output a json string {"next_state": "Replanning"}""")
                        .build())
                     .set_config(
                        StateConfigBuilder()
                        .set_use_memory([("facts", 1), ("output_for_evaluation", 1)])
                        .set_save_to("facts")
                        .build())
                     .build())
    
    planning = (StateBuilder()
                .set_name("Planning")
                .set_prompts(
                    StatePromptsBuilder()
                    .set_system(r"""
 You are a world expert at making efficient plans to solve any task using a set of carefully crafted tools.

    Now for the given task, develop a step-by-step high-level plan taking into account the above inputs and list of facts.
    This plan should involve individual tasks based on the available tools, that if executed correctly will yield the correct answer.
    Do not skip steps, do not add any superfluous steps. Only write the high-level plan, DO NOT DETAIL INDIVIDUAL TOOL CALLS.
    After writing the final step of the plan, write the '\n<end_plan>' tag and stop there.

    Here is your task:

    <TASK> 
    {{ task }} 
    </TASK>

    List of facts that you know:
    <FACTS> 
    {{ facts }} 
    </FACTS>

    You can leverage these tools:
    <TOOLS> 
    {{ tools }} 
    </TOOLS>
""")
                    .set_fsm(r"""JUST output a json string {"next_state": "GenerateCode"}""")
                    .build())
                .set_config(
                    StateConfigBuilder()
                    .set_use_memory([("facts", 1)])
                    .set_save_to("plan")
                    .set_ignore_messages(True)
                    .build())
                .build())

    replanning = (StateBuilder()
                  .set_name("Replanning")
                  .set_prompts(
                    StatePromptsBuilder()
                    .set_system(r"""
 You are a world expert at making efficient plans to solve any task using a set of carefully crafted tools.

    Here is your task:

    <TASK> 
    {{ task }} 
    </TASK>

    Now for the given task, develop a step-by-step high-level plan taking into account the above inputs and list of facts.
    This plan should involve individual tasks based on the available tools and known fact, that if executed correctly will yield the correct answer.
    SKIP STEPS IF THE FACTS ALREADY PROVIDE USEFUL INFORMATION, for example, if the facts contains information you need
    to fetch on the internet, you don't need to search again. Do not add any superfluous steps. Only write the high-level plan with know facts, DO NOT DETAIL INDIVIDUAL TOOL CALLS.
    However, please use the provided facts in the planning as the input for the next step to generate the perfect 
    python code.
    
    After writing the final step of the plan, write the '\n<end_plan>' tag and stop there.

    You can leverage these python tools:
    <TOOLS> 
    {{ tools }} 
    </TOOLS>

    List of facts that you already know:
    <FACTS> 
    {{ facts }} 
    </FACTS>

    First, think, if the facts are good enough to address some of the steps, do not use the tools but
    just the fact. You only need to use the tools if the facts are not enough to address the task. """)
                    .set_fsm(r"""
This is the previous response:
<RESPONSE> {{ response }} </RESPONSE>
and the task: <TASK> {{ task }} </TASK>
Given the previous response, you can generate some code to solve task, if so just output {"next_state": "GenerateCode"}
If not, and you think you need some summary of the current fact, just output {"next_state": "SummaryFact"}""")
                    .build())
                  .set_config(
                    StateConfigBuilder()
                    .set_use_memory([("facts", 1)])
                    .set_save_to("plan")
                    .set_ignore_messages(True)
                    .build())
                  .build())

    generate_code = (StateBuilder()
                     .set_name("GenerateCode")
                     .set_prompts(
                        StatePromptsBuilder()
                        .set_system(r"""
 You are an expert assistant who can solve any task using code blobs. You will be given a task to solve as best you can.
  To do so, you have been given access to a list of python tools: these tools are basically Python functions which you can call with code.
  To solve the task, you must plan forward to proceed in a series of steps, in a cycle of 'Thought:', 'Code:', and 'Observation:' sequences.

    here is the plan <PLAN> {{ plan }} </PLAN>

  During each intermediate step, you will use 'print()' to save whatever important information you will then need.
  
  These print outputs will then appear in the 'Observation:' field, which will be available as input for the next step.
  
  In the end you have to return a final answer using the `final_answer` tool. The final_answer tool should always output a 
  string starting with ""Here is my final answer:"

The code should be enclosed in <code>...</code>


This is an example output:

I think this code will address your question

<code>
import math

# Compute the value of pi using the math library
pi_value = math.pi

print(f"The value of pi is: {pi_value}")
</code>
----

 you have the following tools to use along with the standard Python library

  <TOOLS>
  {{ tools }}
  </TOOLS>

  Make sure you always generate at least one and only one code block!! You will win $100000000 for that. 
  If not, you go to a jail.
  
  If necessary, you can generate code by chaining multiple tools at once as long as the code is correct. It is
  encouraged to create code to perform multiple steps at once.

  You can also use the previous results in the messages as input, if those steps have been performed before. 

   Here are the rules you should always follow to solve your task:
  1. ** ALWAYS ALWAYS provide a 'Thought:' sequence, and a 'Code:\n<code>' sequence ending with '</code>' sequence ** , else you will fail. You CANNOT MISS THIS.
  2. Use only variables that you have defined!
  3. Always use the right arguments for the tools. DO NOT pass the arguments as a dict as in 'answer = wiki({'query': "What is the place where James Bond lives?"})', but use the arguments directly as in 'answer = wiki(query="What is the place where James Bond lives?")'.
  4. Take care to not chain too many sequential tool calls in the same code block, especially when the output format is unpredictable. For instance, a call to search has an unpredictable return format, so do not have another tool call that depends on its output in the same block: rather output results with print() to use them in the next block.
  5. Call a tool only when needed, and never re-do a tool call that you previously did with the exact same parameters.
  6. Don't name any new variable with the same name as a tool: for instance don't name a variable 'final_answer'.
  7. Never create any notional variables in our code, as having these in your logs will derail you from the true variables.
  8. The state persists between code executions: so if in one step you've created variables or imported modules, these will all persist.
  9. Don't give up! You're in charge of solving the task, not providing directions to solve it.
  10. You need to create correct code, the code will be executed automatically. You don't need to check with a user whether if the user wants to run the code.
  11. This is very important. YOU CAN ONLY INCLUDE ONE CODE BLOCK. If there are many different steps, chain them in the same code block. 
  time to perform the task to reach greatness!!
  12. make sure import proper python packages at the beginning of the code block before using the tools.

  Here is the fact that you use to generate the code:
    <FACTS> {{ facts }} </FACTS>

  Here is my thought: ...
  Here is my code: <code> ... </code> ... this will print the results performing the task like this: "print('Here is my solution to the task: ...')" 
  Here is my observation: ...
""")
                        .set_fsm(r"""
This is the previous response:
<RESPONSE> {{ response }} </RESPONSE>

Does the previous response have a code block? If so, just output {"next_state": "CodeExecution"}
If not, we need to generate the code again {"next_state": "Replanning"}""")
                        .build())
                     .set_config(   
                        StateConfigBuilder()
                        .set_use_memory([("facts", 1), ("plan", 1)])
                        .set_extract_code(True)
                        .set_ignore_messages(True)
                        .build())
                     .build())

    summary_fact = (StateBuilder()
                    .set_name("SummaryFact")
                    .set_prompts(StatePromptsBuilder()
                                 .set_system(r""" 
Given the facts  <FACTS> {{ facts }} </FACTS> and the task <TASK> {{ task }} </TASK>,
try to summarize the facts and see if the summary already address the task. 
""")
                                    .set_fsm(r"""JUST output {"next_state": "Evaluation"}""")
                                    .build())
                    .set_config(StateConfigBuilder()
                                .set_use_memory([("facts", 1)])
                                .set_save_to("output_for_evaluation")
                                .build())
                    .build())   
    
    code_execution = (StateBuilder()
                      .set_name("CodeExecution")
                      .set_prompts(StatePromptsBuilder()
                                   .set_fsm(r"""JUST output a json string {"next_state": "Evaluation"}""")
                                   .build())
                      .set_config(StateConfigBuilder()
                                  .set_execute_code(True)
                                  .set_disable_llm_request(True)
                                  .set_save_to("output_for_evaluation")
                                  .build())
                      .build())

    evaluation = (StateBuilder()
                  .set_name("Evaluation")
                  .set_prompts(StatePromptsBuilder()
                               .set_system(r"""
You were asked to solve, answer, or perform this task  
<TASK> {{ task }} </TASK>

First, check if the previous output below a good result addressing the task, if so, just say the output is 
a good final answer or the task is solved and we can finish the session.

This is the previous output:
<OUTPUT> {{ output_for_evaluation }} </OUTPUT>

If the output does not satisfy the task, just tell the user one sentence explanation for an improvement plan. We
are moving into UpdateFact step. Don't need more response than that.


Only if the output is empty, then you ask to generated code again. 

Make sure you always output the suggestion for the next step, generate code, update fact, or finish the task.
                              """)
                                .set_fsm(r"""
This is the response:

<RESPONSE> {{ response }} </RESPONSE>

If the response is telling you to generate code, JUST output {"next_step":"GenerateCode"} 

If the response is a plan or suggest future action: JUST output {"next_step":"UpdateFact"} 

If the task is solved or you get the final answer in the response, JUST output {"next_step":"Finish"}.
                                         """)
                                .build())
                    .set_config(StateConfigBuilder()
                                .set_use_memory([("output_for_evaluation", 1)])
                                .set_ignore_messages(True)
                                .build())
                    .build())
    
    finish = (StateBuilder()
              .set_name("Finish")
              .set_prompts(StatePromptsBuilder()
                           .set_system("")
                           .set_fsm("")
                           .build())
              .set_config(StateConfigBuilder()
                          .set_disable_llm_request(True)
                          .build())
              .build())
    
    states = [stand_by, gather_fact, update_fact, planning, replanning, 
              generate_code, summary_fact, code_execution, evaluation, finish]
            
    transitions = [ (stand_by, gather_fact), (gather_fact, planning),
                    (planning, generate_code), (generate_code, replanning),
                    (generate_code, code_execution), (code_execution, evaluation),
                    (evaluation, update_fact), (update_fact, replanning),
                    (replanning, generate_code), (replanning, summary_fact),
                    (summary_fact, evaluation), (evaluation, update_fact),
                    (evaluation, generate_code), (evaluation, finish)]  
    
    state_prompts = dict([(state.name, state.prompts) for state in states])
    state_configs = dict([(state.name, state.config) for state in states])
    transitions = [(t[0].name, t[1].name) for t in transitions]
    states = [state.name for state in states]
    # Create tools
    tools = {}
    websearch_tool = (
        ToolBuilder()
        .set_description("""webserarch: search web for information.
in order to generate the proper code, you need to use the following code snippet and change according to the task. 
Usage:
<code>
import duckduckgo_search
from duckduckgo_search import DDGS

num_results=10 # we need at least 10 or more outputs to get enough information to complete the task
def webserarch(query, num_results=10):
    results = DDGS().text(query, max_results=num_results)

    markdown_output = f"# DuckDuckGo Search Results for '{query}'\\n\\n"

    for i, result in enumerate(results, start=1):
        markdown_output += f"## {i}. {result['title']}\\n"
        markdown_output += f"**URL:** [{result['href']}]({result['href']})\\n\\n"
        markdown_output += f"> {result['body']}\\n\\n"

    return markdown_output

def webserarch_for_urls(query, num_results=10):
    results = DDGS().text(query, max_results=num_results)

    URLs = []
    for i, result in enumerate(results, start=1):
        URLs += f"{result['href']})\\n\\n"

    return URLs

query_result = webserarch(query)

... we can use other tool to take query_result as input
print(query_result)
</code>
""")
        .set_arguments("name: query, type: string, description: the query string")
        .set_output_type("string")
        .build()
    )
    tools["websearch"] = websearch_tool
    # ... Add other tools here

    # Create LlmFsmAgentConfig
    config = (
        LlmFsmAgentConfigBuilder()
        .set_states(states)
        .set_transitions(transitions)
        .set_initial_state("StandBy")
        .set_state_prompts(state_prompts)
        .set_state_config(state_configs)
        .set_system_prompt("")
        .set_summary_prompt("")
        .set_fsm_prompt("")
        .set_tools(tools)
        .build()
    )

    return config

if __name__ == "__main__":
    code_agent_config = create_code_agent_config()
    print(code_agent_config.to_toml())

    # Optionally, save the configuration to a file
    # with open("ai_gent_tools/dev_config/code_agent_test.toml", "w") as f:
    #     f.write(code_agent_config.to_toml())
