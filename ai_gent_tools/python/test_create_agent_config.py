import unittest
from agent_config import (
    StateConfigBuilder,
    StatePromptsBuilder,
    ToolBuilder,
    LlmFsmAgentConfigBuilder,
    LlmFsmAgentConfig,
)

class TestCreateAgentConfig(unittest.TestCase):

    def test_state_config_builder(self):
        config = StateConfigBuilder()\
            .set_disable_llm_request(True)\
            .set_use_full_context(False)\
            .set_execute_code(True)\
            .set_code("print('Hello, World!')")\
            .build()
        
        self.assertTrue(config.disable_llm_request)
        self.assertFalse(config.use_full_context)
        self.assertTrue(config.execute_code)
        self.assertEqual(config.code, "print('Hello, World!')")

    def test_state_prompts_builder(self):
        prompts = StatePromptsBuilder()\
            .set_system("System prompt")\
            .set_chat("Chat prompt")\
            .set_fsm("FSM prompt")\
            .build()
        
        self.assertEqual(prompts.system, "System prompt")
        self.assertEqual(prompts.chat, "Chat prompt")
        self.assertEqual(prompts.fsm, "FSM prompt")

    def test_tool_builder(self):
        tool = ToolBuilder()\
            .set_description("Test tool")\
            .set_arguments("arg1, arg2")\
            .set_output_type("output")\
            .build()
        
        self.assertEqual(tool.description, "Test tool")
        self.assertEqual(tool.arguments, "arg1, arg2")
        self.assertEqual(tool.output_type, "output")

    def test_llm_fsm_agent_config_builder(self):
        state_prompts = {
            "state1": StatePromptsBuilder().set_system("System 1").build(),
            "state2": StatePromptsBuilder().set_system("System 2").build(),
        }
        
        tools = {
            "tool1": ToolBuilder().set_description("Tool 1").build(),
            "tool2": ToolBuilder().set_description("Tool 2").build(),
        }
        
        config = LlmFsmAgentConfigBuilder()\
            .set_states(["state1", "state2"])\
            .set_transitions([("state1", "state2"), ("state2", "state1")])\
            .set_initial_state("state1")\
            .set_state_prompts(state_prompts)\
            .set_system_prompt("System prompt")\
            .set_summary_prompt("Summary prompt")\
            .set_fsm_prompt("FSM prompt")\
            .set_tools(tools)\
            .build()
        
        self.assertEqual(config.states, ["state1", "state2"])
        self.assertEqual(config.transitions, [("state1", "state2"), ("state2", "state1")])
        self.assertEqual(config.initial_state, "state1")
        self.assertEqual(config.state_prompts, state_prompts)
        self.assertEqual(config.system_prompt, "System prompt")
        self.assertEqual(config.summary_prompt, "Summary prompt")
        self.assertEqual(config.fsm_prompt, "FSM prompt")
        self.assertEqual(config.tools, tools)

    def test_llm_fsm_agent_config_to_json(self):
        config = LlmFsmAgentConfigBuilder()\
            .set_states(["state1"])\
            .set_transitions([("state1", "state1")])\
            .set_initial_state("state1")\
            .set_state_prompts({"state1": StatePromptsBuilder().build()})\
            .set_system_prompt("System prompt")\
            .set_summary_prompt("Summary prompt")\
            .set_fsm_prompt("FSM prompt")\
            .build()
        
        json_str = config.to_json()
        self.assertIsInstance(json_str, str)
        self.assertIn("states", json_str)
        self.assertIn("transitions", json_str)
        self.assertIn("initial_state", json_str)

    def test_llm_fsm_agent_config_to_toml(self):
        config = LlmFsmAgentConfigBuilder()\
            .set_states(["state1"])\
            .set_transitions([("state1", "state1")])\
            .set_initial_state("state1")\
            .set_state_prompts({"state1": StatePromptsBuilder().build()})\
            .set_system_prompt("System prompt")\
            .set_summary_prompt("Summary prompt")\
            .set_fsm_prompt("FSM prompt")\
            .build()
        
        toml_str = config.to_toml()
        self.assertIsInstance(toml_str, str)
        self.assertIn("states", toml_str)
        self.assertIn("transitions", toml_str)
        self.assertIn("initial_state", toml_str)
        
class TestLlmFsmAgentConfigBuilder(unittest.TestCase):

    def setUp(self):
        self.builder = LlmFsmAgentConfigBuilder()

    def test_empty_states_validation(self):
        with self.assertRaisesRegex(ValueError, "States cannot be empty"):
            self.builder.build()

    def test_initial_state_validation(self):
        self.builder.set_states(["state1", "state2"])
        self.builder.set_initial_state("invalid_state")
        with self.assertRaisesRegex(ValueError, "Initial state 'invalid_state' must be one of the defined states"):
            self.builder.build()

    def test_invalid_transition_format(self):
        self.builder.set_states(["state1", "state2"])
        self.builder.set_initial_state("state1")
        self.builder.set_transitions(["invalid_transition"])
        with self.assertRaisesRegex(ValueError, "Invalid transition format:*"):
            self.builder.build()

    def test_invalid_transition_states(self):
        self.builder.set_states(["state1", "state2"])
        self.builder.set_initial_state("state1")
        self.builder.set_transitions([["state1", "invalid_state"]])
        with self.assertRaisesRegex(ValueError, "Invalid transition:.*'invalid_state' is not in the defined states"):
            self.builder.build()

    def test_valid_configuration(self):
        self.builder.set_states(["state1", "state2"])
        self.builder.set_initial_state("state1")
        self.builder.set_transitions([["state1", "state2"], ["state2", "state1"]])
        self.builder.set_state_prompts({})
        self.builder.set_system_prompt("system prompt")
        self.builder.set_summary_prompt("summary prompt")
        self.builder.set_fsm_prompt("fsm prompt")

        config = self.builder.build()
        self.assertIsNotNone(config)

    def test_valid_config_attributes(self):
        self.builder.set_states(["state1", "state2"])
        self.builder.set_initial_state("state1")
        self.builder.set_transitions([("state1", "state2"), ("state2", "state1")])
        self.builder.set_state_prompts({})
        self.builder.set_system_prompt("system prompt")
        self.builder.set_summary_prompt("summary prompt")
        self.builder.set_fsm_prompt("fsm prompt")

        config = self.builder.build()
        self.assertEqual(config.states, ["state1", "state2"])
        self.assertEqual(config.initial_state, "state1")
        self.assertEqual(config.transitions, [("state1", "state2"), ("state2", "state1")])
        self.assertEqual(config.system_prompt, "system prompt")
        self.assertEqual(config.summary_prompt, "summary prompt")
        self.assertEqual(config.fsm_prompt, "fsm prompt")


if __name__ == '__main__':
    unittest.main()
