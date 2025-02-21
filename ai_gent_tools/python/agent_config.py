from typing import Optional, List, Dict, Tuple
from pydantic import BaseModel
import json
import toml

class StateConfig(BaseModel):
    disable_llm_request: Optional[bool] = None
    use_full_context: Optional[bool] = None
    use_only_last_message: Optional[bool] = None
    ignore_llm_output: Optional[bool] = None
    ignore_messages: Optional[bool] = None
    save_to_summary: Optional[bool] = None
    save_to_context: Optional[bool] = None
    save_execution_output: Optional[bool] = None
    extract_code: Optional[bool] = None
    execute_code: Optional[bool] = None
    code: Optional[str] = None
    fsm_code: Optional[str] = None
    wait_for_msg: Optional[bool] = None
    save_to: Optional[List[str]] = None
    use_memory: Optional[List[Tuple[str, int]]] = None

class StateConfigBuilder:
    def __init__(self):
        self._config = StateConfig()
    
    def set_disable_llm_request(self, value: bool):
        self._config.disable_llm_request = value
        return self
    
    def set_use_full_context(self, value: bool):
        self._config.use_full_context = value
        return self
    
    def set_use_only_last_message(self, value: bool):
        self._config.use_only_last_message = value
        return self
    
    def set_ignore_llm_output(self, value: bool):
        self._config.ignore_llm_output = value
        return self
    
    def set_ignore_messages(self, value: bool):
        self._config.ignore_messages = value
        return self
    
    def set_save_to_summary(self, value: bool):
        self._config.save_to_summary = value
        return self
    
    def set_save_to_context(self, value: bool):
        self._config.save_to_context = value
        return self
    
    def set_save_execution_output(self, value: bool):
        self._config.save_execution_output = value
        return self
    
    def set_extract_code(self, value: bool):
        self._config.extract_code = value
        return self
    
    def set_execute_code(self, value: bool):
        self._config.execute_code = value
        return self
    
    def set_code(self, value: str):
        self._config.code = value
        return self
    
    def set_fsm_code(self, value: str):
        self._config.fsm_code = value
        return self
    
    def set_wait_for_msg(self, value: bool):
        self._config.wait_for_msg = value
        return self
    
    def set_save_to(self, value: List[str]):
        self._config.save_to = value
        return self
    
    def set_use_memory(self, value: List[Tuple[str, int]]):
        self._config.use_memory = value
        return self
    
    def build(self):
        return self._config

class StatePrompts(BaseModel):
    system: Optional[str] = None
    chat: Optional[str] = None
    fsm: Optional[str] = None

class StatePromptsBuilder:
    def __init__(self):
        self._prompts = StatePrompts()
    
    def set_system(self, value: str):
        self._prompts.system = value
        return self
    
    def set_chat(self, value: str):
        self._prompts.chat = value
        return self
    
    def set_fsm(self, value: str):
        self._prompts.fsm = value
        return self
    
    def build(self):
        return self._prompts

class Tool(BaseModel):
    description: str
    arguments: str
    output_type: str

class ToolBuilder:
    def __init__(self):
        self.description = ""
        self.arguments = ""
        self.output_type = ""

    def set_description(self, description: str):
        self.description = description
        return self

    def set_arguments(self, arguments: str):
        self.arguments = arguments
        return self

    def set_output_type(self, output_type: str):
        self.output_type = output_type
        return self

    def build(self) -> Tool:
        return Tool(
            description=self.description,
            arguments=self.arguments,
            output_type=self.output_type
        )

class LlmFsmAgentConfig(BaseModel):
    states: List[str]
    transitions: List[Tuple[str, str]]
    initial_state: str
    state_prompts: Dict[str, StatePrompts]
    state_config: Optional[Dict[str, StateConfig]] = None
    system_prompt: str
    summary_prompt: str
    fsm_prompt: str
    tools: Optional[Dict[str, Tool]] = None

    @classmethod
    def from_dict(cls, data: dict):
        return cls(**data)

    def to_json(self) -> str:
        return self.json()
    
    def to_toml(self) -> str:
        return toml.dumps(self.model_dump())

    @classmethod
    def from_toml(cls, file_path: str):
        with open(file_path, 'r') as toml_file:
            data = toml.load(toml_file)
        return cls(**data)
    
    @classmethod
    def from_json(cls, file_path: str):
        with open(file_path, 'r') as json_file:
            data = json.load(json_file)
        return cls(**data)



class LlmFsmAgentConfigBuilder:
    def __init__(self):
        self._states = []
        self._transitions = []
        self._initial_state = ""
        self._state_prompts = {}
        self._state_config = None
        self._system_prompt = ""
        self._summary_prompt = ""
        self._fsm_prompt = ""
        self._tools = None

    def set_states(self, states: List[str]):
        self._states = states
        return self

    def set_transitions(self, transitions: List[Tuple[str, str]]):
        self._transitions = transitions
        return self

    def set_initial_state(self, initial_state: str):
        self._initial_state = initial_state
        return self

    def set_state_prompts(self, state_prompts: Dict[str, StatePrompts]):
        self._state_prompts = state_prompts
        return self

    def set_state_config(self, state_config: Optional[Dict[str, StateConfig]]):
        self._state_config = state_config
        return self

    def set_system_prompt(self, system_prompt: str):
        self._system_prompt = system_prompt
        return self

    def set_summary_prompt(self, summary_prompt: str):
        self._summary_prompt = summary_prompt
        return self

    def set_fsm_prompt(self, fsm_prompt: str):
        self._fsm_prompt = fsm_prompt
        return self

    def set_tools(self, tools: Optional[Dict[str, Tool]]):
        self._tools = tools
        return self

    def build(self):
        if not self._states:
            raise ValueError("States cannot be empty")
        
        if self._initial_state not in self._states:
            raise ValueError(f"Initial state '{self._initial_state}' must be one of the defined states: {self._states}")

        for transition in self._transitions:
            if not isinstance(transition, (list, tuple)) or len(transition) != 2:
                raise ValueError(f"Invalid transition format: {transition}. Expected [state1, state2]")
            state1, state2 = transition
            if state1 not in self._states:
                raise ValueError(f"Invalid transition: {transition}. '{state1}' is not in the defined states: {self._states}")
            if state2 not in self._states:
                raise ValueError(f"Invalid transition: {transition}. '{state2}' is not in the defined states: {self._states}")
            
        return LlmFsmAgentConfig(
            states=self._states,
            transitions=self._transitions,
            initial_state=self._initial_state,
            state_prompts=self._state_prompts,
            state_config=self._state_config,
            system_prompt=self._system_prompt,
            summary_prompt=self._summary_prompt,
            fsm_prompt=self._fsm_prompt,
            tools=self._tools
        )

class State(BaseModel):
    name: str
    prompts: Optional[StatePrompts] = None
    config: Optional[StateConfig] = None

class StateBuilder:
    def __init__(self):
        self._state = State(name="")
        self._prompts_builder = StatePromptsBuilder()
        self._config_builder = StateConfigBuilder()
    
    def set_name(self, name: str):
        self._state.name = name
        return self
    
    def set_prompts(self, prompts: StatePrompts):
        self._state.prompts = prompts
        return self
    
    def set_config(self, config: StateConfig):
        self._state.config = config
        return self
    
    def build(self):
        if not self._state.prompts:
            self._state.prompts = self._prompts_builder.build()
        if not self._state.config:
            self._state.config = self._config_builder.build()
        return self._state


def create_state_dictionaries(states: List[State]) -> Tuple[Dict[str, StatePrompts], Dict[str, StateConfig]]:
    prompt_dict: Dict[str, StatePrompts] = {}
    config_dict: Dict[str, StateConfig] = {}

    for state in states:
        if state.prompts:
            prompt_dict[state.name] = state.prompts
        if state.config:
            config_dict[state.name] = state.config

    return prompt_dict, config_dict


def create_example_llm_fsm_agent_config() -> None:
    # Create State 1: Initial
    initial_state = (
        StateBuilder()
        .set_name("Initial")
        .set_prompts(
            StatePromptsBuilder()
            .set_system("You are an AI assistant in the initial state.")
            .set_chat("How can I help you today?")
            .build()
        )
        .set_config(
            StateConfigBuilder()
            .set_use_full_context(True)
            .set_wait_for_msg(True)
            .build()
        )
        .build()
    )

    # Create State 2: Processing
    processing_state = (
        StateBuilder()
        .set_name("Processing")
        .set_prompts(
            StatePromptsBuilder()
            .set_system("You are now processing the user's request.")
            .set_chat("I'm working on your request. Please provide any additional details if needed.")
            .build()
        )
        .set_config(
            StateConfigBuilder()
            .set_use_full_context(True)
            .set_extract_code(True)
            .set_execute_code(True)
            .build()
        )
        .build()
    )

    # Create State 3: Finalizing
    finalizing_state = (
        StateBuilder()
        .set_name("Finalizing")
        .set_prompts(
            StatePromptsBuilder()
            .set_system("You are finalizing the response to the user's request.")
            .set_chat("I've completed processing. Here's what I found:")
            .build()
        )
        .set_config(
            StateConfigBuilder()
            .set_use_full_context(True)
            .set_save_to_summary(True)
            .build()
        )
        .build()
    )

    # List of states
    states = [initial_state, processing_state, finalizing_state]

    # Create state prompts and configs using the provided function
    state_prompts, state_configs = create_state_dictionaries(states)

    # Create a tool
    weather_tool = (
        ToolBuilder()
        .set_description("Get the current weather for a given location")
        .set_arguments("location: str - The city or location to get weather for")
        .set_output_type("A string describing the current weather conditions")
        .build()
    )

    # Create LlmFsmAgentConfig using the builder
    llm_fsm_agent_config = (
        LlmFsmAgentConfigBuilder()
        .set_states([state.name for state in states])
        .set_transitions([("Initial", "Processing"), ("Processing", "Finalizing")])
        .set_initial_state("Initial")
        .set_state_prompts(state_prompts)
        .set_state_config(state_configs)
        .set_system_prompt("You are an AI agent with multiple states. Follow the instructions for each state.")
        .set_summary_prompt("Summarize the actions taken in each state.")
        .set_fsm_prompt("Decide which state to transition to next based on the current state and input.")
        .set_tools({"weather": weather_tool})
        .build()
    )
    
    print(llm_fsm_agent_config.to_toml())
    

if __name__ == "__main__":
    create_example_llm_fsm_agent_config()
    test = LlmFsmAgentConfig.from_toml("../dev_config/code_agent.toml")
    #print(test.to_json())
    print(test.to_toml())