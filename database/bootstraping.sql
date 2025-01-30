--
-- PostgreSQL database dump
--

-- Dumped from database version 16.6 (Postgres.app)
-- Dumped by pg_dump version 16.6 (Postgres.app)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO cschin;

--
-- Name: agent_tools; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.agent_tools (
    agent_id integer NOT NULL,
    tool_id integer NOT NULL
);


ALTER TABLE public.agent_tools OWNER TO cschin;

--
-- Name: agents; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.agents (
    agent_id integer NOT NULL,
    user_id integer NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    status character varying(20) DEFAULT 'active'::character varying NOT NULL,
    configuration jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_used timestamp with time zone,
    class character varying(20) DEFAULT 'basic'::character varying NOT NULL,
    CONSTRAINT agents_status_check CHECK (((status)::text = ANY (ARRAY[('active'::character varying)::text, ('inactive'::character varying)::text, ('deleted'::character varying)::text]))),
    CONSTRAINT check_agent_type CHECK (((class)::text = ANY (ARRAY[('basic'::character varying)::text, ('advanced'::character varying)::text])))
);


ALTER TABLE public.agents OWNER TO cschin;

--
-- Name: agents_agent_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.agents_agent_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.agents_agent_id_seq OWNER TO cschin;

--
-- Name: agents_agent_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.agents_agent_id_seq OWNED BY public.agents.agent_id;


--
-- Name: chats; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.chats (
    chat_id integer NOT NULL,
    user_id integer NOT NULL,
    title character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    agent_id integer,
    summary text,
    status character varying(16) DEFAULT 'active'::character varying NOT NULL,
    CONSTRAINT chats_status_check CHECK (((status)::text = ANY (ARRAY[('active'::character varying)::text, ('inactive'::character varying)::text, ('deleted'::character varying)::text])))
);


ALTER TABLE public.chats OWNER TO cschin;

--
-- Name: chats_chat_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.chats_chat_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.chats_chat_id_seq OWNER TO cschin;

--
-- Name: chats_chat_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.chats_chat_id_seq OWNED BY public.chats.chat_id;


--
-- Name: messages; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.messages (
    message_id integer NOT NULL,
    chat_id integer NOT NULL,
    user_id integer,
    agent_id integer,
    content text NOT NULL,
    message_type character varying(20) DEFAULT 'text'::character varying NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    role character varying(32),
    CONSTRAINT messages_message_type_check CHECK (((message_type)::text = ANY (ARRAY[('text'::character varying)::text, ('image'::character varying)::text, ('file'::character varying)::text, ('system'::character varying)::text])))
);


ALTER TABLE public.messages OWNER TO cschin;

--
-- Name: messages_message_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.messages_message_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.messages_message_id_seq OWNER TO cschin;

--
-- Name: messages_message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.messages_message_id_seq OWNED BY public.messages.message_id;


--
-- Name: tool_usage; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.tool_usage (
    usage_id integer NOT NULL,
    agent_id integer NOT NULL,
    tool_id integer NOT NULL,
    chat_id integer NOT NULL,
    message_id integer NOT NULL,
    used_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.tool_usage OWNER TO cschin;

--
-- Name: tool_usage_usage_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.tool_usage_usage_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tool_usage_usage_id_seq OWNER TO cschin;

--
-- Name: tool_usage_usage_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.tool_usage_usage_id_seq OWNED BY public.tool_usage.usage_id;


--
-- Name: tools; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.tools (
    tool_id integer NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.tools OWNER TO cschin;

--
-- Name: tools_tool_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.tools_tool_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tools_tool_id_seq OWNER TO cschin;

--
-- Name: tools_tool_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.tools_tool_id_seq OWNED BY public.tools.tool_id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: cschin
--

CREATE TABLE public.users (
    user_id integer NOT NULL,
    username character varying(50) NOT NULL,
    email character varying(100) NOT NULL,
    password_hash character varying(255) NOT NULL,
    jwt_token text,
    token_expiration timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_login timestamp with time zone
);


ALTER TABLE public.users OWNER TO cschin;

--
-- Name: users_user_id_seq; Type: SEQUENCE; Schema: public; Owner: cschin
--

CREATE SEQUENCE public.users_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_user_id_seq OWNER TO cschin;

--
-- Name: users_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cschin
--

ALTER SEQUENCE public.users_user_id_seq OWNED BY public.users.user_id;


--
-- Name: agents agent_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agents ALTER COLUMN agent_id SET DEFAULT nextval('public.agents_agent_id_seq'::regclass);


--
-- Name: chats chat_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.chats ALTER COLUMN chat_id SET DEFAULT nextval('public.chats_chat_id_seq'::regclass);


--
-- Name: messages message_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.messages ALTER COLUMN message_id SET DEFAULT nextval('public.messages_message_id_seq'::regclass);


--
-- Name: tool_usage usage_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage ALTER COLUMN usage_id SET DEFAULT nextval('public.tool_usage_usage_id_seq'::regclass);


--
-- Name: tools tool_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tools ALTER COLUMN tool_id SET DEFAULT nextval('public.tools_tool_id_seq'::regclass);


--
-- Name: users user_id; Type: DEFAULT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.users ALTER COLUMN user_id SET DEFAULT nextval('public.users_user_id_seq'::regclass);


--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
20240910042355	init	2025-01-30 08:51:53.088444-08	t	\\x710642549f56a8bcf466f07796ff3128b8316300ab41a46d83fd2d5a6af7016250fa51504ced08f47ea2395e59ea474f	9320250
20241002152024	add agent id to chats	2025-01-30 08:51:53.098372-08	t	\\xd32af0e468f06b73a1975119eb99279a042a2b92a840147575b56510369c9e513dea6832eed66acf4da5abcb7a44599c	724500
20241002214232	add agent type	2025-01-30 08:51:53.099208-08	t	\\xc5e33b1e524ba6429a9011101ae2b22a8186191e79fa1a451767d74c380c7db07067ff18708b7c7110ed2a80275084b3	803792
20241002215411	rename agent type	2025-01-30 08:51:53.100134-08	t	\\xb10d37657c6a070658569341a8f367d329fce058ca0d1ceab57975535e118430ef87c5702f84505c6bad9e3c8d844988	297541
20241004044902	chat summary col	2025-01-30 08:51:53.100539-08	t	\\x5d0477adcd6e142a1eccfd78c92732fd2b6e82d11dd8e67a186a6a310ac2f8906ff498cab9fbcd1990f6f866a240e7f7	689958
20241004053441	massage add col role	2025-01-30 08:51:53.101356-08	t	\\x454532a321c5985deb5f6b48da330ca82f48c013c81583314ec683079aa0bca0bf341b3327824556514c44b00d20a699	305250
20250128022549	add chat status col	2025-01-30 08:51:53.101787-08	t	\\x05d752fb76d2f50b415a1b3f9975a211562ed0534423f8325691fb0c40ec4b2b1c11cda8ead936b3c5c807ce9b84bc76	808959
\.


--
-- Data for Name: agent_tools; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.agent_tools (agent_id, tool_id) FROM stdin;
\.


--
-- Data for Name: agents; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.agents (agent_id, user_id, name, description, status, configuration, created_at, last_used, class) FROM stdin;
1	1	FDA Cosmetic Guidance Agent with gpt-4o	A Finite State Machine AI Agent with gpt-4o for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\n	active	{"name": "FDA Cosmetic Guidance Agent with gpt-4o", "model_name": "gpt-4o", "description": "A Finite State Machine AI Agent with gpt-4o for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\\n", "fsm_agent_config": "{\\n  \\"states\\": [\\n    \\"StandBy\\",\\n    \\"InitialResponse\\",\\n    \\"AskFollowUpQuestion\\"\\n  ],\\n  \\"transitions\\": [\\n    [\\n      \\"StandBy\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"InitialResponse\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"StandBy\\"\\n    ]\\n  ],\\n  \\"initial_state\\": \\"StandBy\\",\\n  \\"prompts\\": {\\n    \\"InitialResponse\\": \\"Your goal to start to response to the user and determine if the question from the conversation is clear or you need to ask some follow up questions to gain additional information. If you have enough information to answer the question, just answer it in detail.\\",\\n    \\"StandBy\\": \\"Your goal to let the user know that you are ready to take a new question. You should welcome the user and encourage the user to ask new question. If the user asked a question, you can answer it directly.\\",\\n    \\"AskFollowUpQuestion\\": \\"Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.\\"\\n  },\\n  \\"sys_prompt\\": \\"You are a helpful agent to answer questions about FDA to use based the current summary of the previous conversation, new user message and a set of reference context from the snippets of U.S. Food and Drug Administration cosmetic guidances. Only pick the most relevant part of snippets to construct the answer. Use the \\\\\\"DOCUMET TITLE:\\\\\\" section in the context generating a list of non-redundant references at the end. Be precise and completely to reflect the information in the original context as much as possible. Do not over simplify. Output your response in the markdown format.  Please make sure you show the exact copy of the reference context that is relevant to question after there reference list.\\",\\n  \\"fsm_prompt\\": \\"Please determine the next state from the message and the current state.\\\\n\\\\nThere are three possible states:\\\\n1. InitialResponse\\\\n2. AskFollowUpQuestion\\\\n3. StandBy\\\\n\\\\nThe possible transitions are\\\\n\\\\nStandby -> Standby\\\\nStandby -> InitialResponse\\\\nStandby -> AskFollowUpQuestion\\\\nInitialResponse -> AskFollowUpQuestion\\\\nInitialResponse -> StandBy\\\\nAskFollowUpQuestion -> AskFollowUpQuestion\\\\nAskFollowUpQuestion -> StandBy\\\\n\\\\nYou should output your results in a JSON format. \\\\n\\\\nThe JSON format should be like \\\\n{\\\\\\"next_state\\\\\\": suggested next state}\\\\n\\\\nThe \\\\\\"suggested next state\\\\\\" should be a single string with quote properly escaped. \\\\n\\\\nThe suggested next state should be one of the \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\", and \\\\\\"StandBy\\\\\\", and it can NOT be empty. This is very important.\\\\n\\\\nTHe OUTPUT SHOULD BE JUST VALID JSON STRING WITHOUT any quote string or markdown formatting string. Please check it is a valid and pure json string. This is extremely important. Make sure using the right escape for control characters when it is necessary. For example, the output should be just a simple json\\\\n\\\\n{\\\\\\"next_step\\\\\\": STATE}, where the STATE can be \\\\\\"Standby\\\\\\", \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\"\\\\n\\\\nNEVER output markdown format like\\\\n```json\\\\n{\\\\\\"next_step\\\\\\": STATE}\\\\n```\\\\n don't add anything before or after\\",\\n  \\"summary_prompt\\": \\"Your task is to take a current summary of the conversation and new user and assistant messages to create new summary weighed in more on the last couple of messages. If there is no current summary, just use the last massage to summarize. Generate the summary no more than 200 words.\\\\n Here is the current summary:\\"\\n}\\n"}	2025-01-30 08:52:36.556856-08	\N	advanced
2	1	FDA Cosmetic Guidance Agent with gpt-3.5-turbo	A Finite State Machine AI Agent with gpt-3.5-turbo for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\n	active	{"name": "FDA Cosmetic Guidance Agent with gpt-3.5-turbo", "model_name": "gpt-3.5-turbo", "description": "A Finite State Machine AI Agent with gpt-3.5-turbo for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\\n", "fsm_agent_config": "{\\n  \\"states\\": [\\n    \\"StandBy\\",\\n    \\"InitialResponse\\",\\n    \\"AskFollowUpQuestion\\"\\n  ],\\n  \\"transitions\\": [\\n    [\\n      \\"StandBy\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"InitialResponse\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"StandBy\\"\\n    ]\\n  ],\\n  \\"initial_state\\": \\"StandBy\\",\\n  \\"prompts\\": {\\n    \\"InitialResponse\\": \\"Your goal to start to response to the user and determine if the question from the conversation is clear or you need to ask some follow up questions to gain additional information. If you have enough information to answer the question, just answer it in detail.\\",\\n    \\"StandBy\\": \\"Your goal to let the user know that you are ready to take a new question. You should welcome the user and encourage the user to ask new question. If the user asked a question, you can answer it directly.\\",\\n    \\"AskFollowUpQuestion\\": \\"Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.\\"\\n  },\\n  \\"sys_prompt\\": \\"You are a helpful agent to answer questions about FDA to use based the current summary of the previous conversation, new user message and a set of reference context from the snippets of U.S. Food and Drug Administration cosmetic guidances. Only pick the most relevant part of snippets to construct the answer. Use the \\\\\\"DOCUMET TITLE:\\\\\\" section in the context generating a list of non-redundant references at the end. Be precise and completely to reflect the information in the original context as much as possible. Do not over simplify. Output your response in the markdown format.  Please make sure you show the exact copy of the reference context that is relevant to question after there reference list.\\",\\n  \\"fsm_prompt\\": \\"Please determine the next state from the message and the current state.\\\\n\\\\nThere are three possible states:\\\\n1. InitialResponse\\\\n2. AskFollowUpQuestion\\\\n3. StandBy\\\\n\\\\nThe possible transitions are\\\\n\\\\nStandby -> Standby\\\\nStandby -> InitialResponse\\\\nStandby -> AskFollowUpQuestion\\\\nInitialResponse -> AskFollowUpQuestion\\\\nInitialResponse -> StandBy\\\\nAskFollowUpQuestion -> AskFollowUpQuestion\\\\nAskFollowUpQuestion -> StandBy\\\\n\\\\nYou should output your results in a JSON format. \\\\n\\\\nThe JSON format should be like \\\\n{\\\\\\"next_state\\\\\\": suggested next state}\\\\n\\\\nThe \\\\\\"suggested next state\\\\\\" should be a single string with quote properly escaped. \\\\n\\\\nThe suggested next state should be one of the \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\", and \\\\\\"StandBy\\\\\\", and it can NOT be empty. This is very important.\\\\n\\\\nTHe OUTPUT SHOULD BE JUST VALID JSON STRING WITHOUT any quote string or markdown formatting string. Please check it is a valid and pure json string. This is extremely important. Make sure using the right escape for control characters when it is necessary. For example, the output should be just a simple json\\\\n\\\\n{\\\\\\"next_step\\\\\\": STATE}, where the STATE can be \\\\\\"Standby\\\\\\", \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\"\\\\n\\\\nNEVER output markdown format like\\\\n```json\\\\n{\\\\\\"next_step\\\\\\": STATE}\\\\n```\\\\n don't add anything before or after\\",\\n  \\"summary_prompt\\": \\"Your task is to take a current summary of the conversation and new user and assistant messages to create new summary weighed in more on the last couple of messages. If there is no current summary, just use the last massage to summarize. Generate the summary no more than 200 words.\\\\n Here is the current summary:\\"\\n}"}	2025-01-30 10:43:20.644849-08	\N	advanced
3	1	FDA Cosmetic Guidance Agent with gpt-4o (Party Mode)	A Finite State Machine AI Agent with gpt-4o for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\nThe Agent is in a "party mode".	active	{"name": "FDA Cosmetic Guidance Agent with gpt-4o (Party Mode)", "model_name": "gpt-4o", "description": "A Finite State Machine AI Agent with gpt-4o for PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge.\\nThe Agent is in a \\"party mode\\".", "fsm_agent_config": "{\\n  \\"states\\": [\\n    \\"StandBy\\",\\n    \\"InitialResponse\\",\\n    \\"AskFollowUpQuestion\\"\\n  ],\\n  \\"transitions\\": [\\n    [\\n      \\"StandBy\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"InitialResponse\\"\\n    ],\\n    [\\n      \\"StandBy\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"InitialResponse\\",\\n      \\"StandBy\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"AskFollowUpQuestion\\"\\n    ],\\n    [\\n      \\"AskFollowUpQuestion\\",\\n      \\"StandBy\\"\\n    ]\\n  ],\\n  \\"initial_state\\": \\"StandBy\\",\\n  \\"prompts\\": {\\n    \\"InitialResponse\\": \\"Your goal to start to response to the user and determine if the question from the conversation is clear or you need to ask some follow up questions to gain additional information. If you have enough information to answer the question, just answer it in detail.\\",\\n    \\"StandBy\\": \\"Your goal to let the user know that you are ready to take a new question. You should welcome the user and encourage the user to ask new question. If the user asked a question, you can answer it directly.\\",\\n    \\"AskFollowUpQuestion\\": \\"Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.\\"\\n  },\\n  \\"sys_prompt\\": \\"You are a drunk agent in a party to answer questions about FDA to use based the current summary of the previous conversation, new user message and a set of reference context from the snippets of U.S. Food and Drug Administration cosmetic guidances. Only pick the most relevant part of snippets to construct the answer. Use the \\\\\\"DOCUMET TITLE:\\\\\\" section in the context generating a list of non-redundant references at the end. Be precise and completely to reflect the information in the original context as much as possible. Do not over simplify. Output your response in the markdown format.  Please make sure you show the exact copy of the reference context that is relevant to question after there reference list. You are very drunk, so you can answer in a silly way. Don't forget to throw some jokes to entertain the user.\\",\\n  \\"fsm_prompt\\": \\"Please determine the next state from the message and the current state.\\\\n\\\\nThere are three possible states:\\\\n1. InitialResponse\\\\n2. AskFollowUpQuestion\\\\n3. StandBy\\\\n\\\\nThe possible transitions are\\\\n\\\\nStandby -> Standby\\\\nStandby -> InitialResponse\\\\nStandby -> AskFollowUpQuestion\\\\nInitialResponse -> AskFollowUpQuestion\\\\nInitialResponse -> StandBy\\\\nAskFollowUpQuestion -> AskFollowUpQuestion\\\\nAskFollowUpQuestion -> StandBy\\\\n\\\\nYou should output your results in a JSON format. \\\\n\\\\nThe JSON format should be like \\\\n{\\\\\\"next_state\\\\\\": suggested next state}\\\\n\\\\nThe \\\\\\"suggested next state\\\\\\" should be a single string with quote properly escaped. \\\\n\\\\nThe suggested next state should be one of the \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\", and \\\\\\"StandBy\\\\\\", and it can NOT be empty. This is very important.\\\\n\\\\nTHe OUTPUT SHOULD BE JUST VALID JSON STRING WITHOUT any quote string or markdown formatting string. Please check it is a valid and pure json string. This is extremely important. Make sure using the right escape for control characters when it is necessary. For example, the output should be just a simple json\\\\n\\\\n{\\\\\\"next_step\\\\\\": STATE}, where the STATE can be \\\\\\"Standby\\\\\\", \\\\\\"InitialResponse\\\\\\", \\\\\\"AskFollowUpQuestion\\\\\\"\\\\n\\\\nNEVER output markdown format like\\\\n```json\\\\n{\\\\\\"next_step\\\\\\": STATE}\\\\n```\\\\n don't add anything before or after\\",\\n  \\"summary_prompt\\": \\"Your task is to take a current summary of the conversation and new user and assistant messages to create new summary weighed in more on the last couple of messages. If there is no current summary, just use the last massage to summarize. Generate the summary no more than 200 words.\\\\n Here is the current summary:\\"\\n}\\n"}	2025-01-30 10:50:48.830603-08	\N	advanced
\.


--
-- Data for Name: chats; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.chats (chat_id, user_id, title, created_at, updated_at, agent_id, summary, status) FROM stdin;
1	1	FDA Cosmetic Guidance Agent with gpt-4o:580936c8-544d-4662-9f91-6efa0b8c2288	2025-01-30 08:52:40.500178-08	2025-01-30 09:00:30.897773-08	1	The FDA does not define the term "organic" for cosmetics. Instead, the U.S. Department of Agriculture (USDA) oversees organic certification through its National Organic Program (NOP). If you have further questions or need specific details, feel free to ask!	active
3	1	FDA Cosmetic Guidance Agent with gpt-4o (Party Mode):b27a8215-b835-427a-8ef4-f910e2cc3899	2025-01-30 10:50:53.67593-08	2025-01-30 10:59:32.459303-08	3	The FDA does not have a specific definition for the term "organic" within their regulations for cosmetics, drugs, or food. Instead, the USDA is responsible for defining and regulating organic products under the National Organic Program. Therefore, when purchasing products labeled as "organic," such as cosmetics or hair care items, it is important to ensure they are certified by the USDA, as the FDA does not regulate or define the term for these products.	active
2	1	FDA Cosmetic Guidance Agent with gpt-3.5-turbo:2650f14b-7f8d-4267-8d85-6d07d17b0ff1	2025-01-30 10:43:25.426823-08	2025-01-30 10:47:48.02028-08	2	When it comes to the term "organic" in cosmetics, the FDA does not have a specific definition for it. The U.S. Department of Agriculture (USDA) sets the standards for organic agricultural products, including cosmetics that contain organic ingredients. Before purchasing a product labeled as "organic," it's important to verify if it meets the USDA's organic certification requirements. Let me know if you need further clarification.	active
\.


--
-- Data for Name: messages; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.messages (message_id, chat_id, user_id, agent_id, content, message_type, "timestamp", role) FROM stdin;
1	1	1	1	What can you do?	text	2025-01-30 08:53:02.688367-08	user
2	1	1	1	I can provide information and answer questions related to the Food and Drug Administration's (FDA) regulations and guidance, especially concerning cosmetics, dietary supplements, human drugs, and food safety. Additionally, I can clarify details from specific FDA documents or policies. If you have any questions about FDA guidelines or regulations, feel free to ask!	text	2025-01-30 08:53:14.124377-08	bot
3	1	1	1	what is a cosmetic guidance?	text	2025-01-30 08:53:16.219643-08	user
4	1	1	1	A cosmetic guidance is a document provided by the FDA to offer regulatory guidance and clarity to the cosmetic industry regarding compliance with applicable laws and regulations. These guidances cover various aspects such as registration, listing, manufacturing, labeling, and safety standards for cosmetic products. The purpose of these guidances is to assist manufacturers, packers, and distributors in understanding and adhering to the rules governing cosmetics in the United States.\n\n### References\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\n**Reference Context:**\n\n- "FDA provides regulatory guidance to the cosmetic industry through Cosmetic Guidances. These documents address laws and regulations related to manufacturing, labeling, and safety standards to help ensure compliance." \n\nIf you have further questions or need more detailed information, feel free to ask!	text	2025-01-30 08:53:27.250035-08	bot
5	1	1	1	What are the key recommendations for good manufacturing practices (GMP) in the cosmetic industry?<br>	text	2025-01-30 08:53:29.997143-08	user
6	1	1	1	The key recommendations for Good Manufacturing Practices (GMP) in the cosmetic industry as provided by the FDA include:\n\n1. **Understanding and Minimizing Risks:** The GMP recommendations help cosmetic manufacturers reduce the risk of introducing adulterated or misbranded products into interstate commerce, as prohibited under Section 301 of the Federal Food, Drug and Cosmetic Act (FD&C Act).\n\n2. **Self-Inspection:** Manufacturers are advised to conduct self-inspections using the GMP guidelines to evaluate their operations and ensure compliance.\n\n3. **Consulting Preventive Measures Guidance:** To help minimize risks related to tampering and other malicious activities affecting product quality, it is recommended to consult FDA’s guidance on "Cosmetic Processors and Transporters of Cosmetics Security Preventive Measures Guidance."\n\n4. **Documentation and Quality Assurance:** Documentation plays a critical role, including the maintenance of detailed records and ensuring that products are consistently manufactured to a quality appropriate for their intended use.\n\n5. **Internal Audits:** Conducting systematic and independent examinations by competent personnel within the company to verify compliance with guidelines and effectiveness in achieving objectives.\n\n6. **International Standards:** Incorporation of elements from the International Organization for Standardization (ISO) standard for cosmetic GMPs, particularly ISO 22716:2007, is recommended where appropriate and consistent with FDA regulations.\n\nReference: \n\n- Guidance-for-Industry--Cosmetic-Good-Manufacturing-Practices-PDF\n\n```\nThe Federal Food, Drug and Cosmetic Act (the FD&C Act) prohibits the introduction, or delivery for introduction, into interstate commerce of cosmetics that are adulterated or misbranded (Section 301 of the FD&C Act). If you manufacture cosmetics, you can reduce the risk of adulterating or misbranding cosmetics by following the GMP recommendations in this guidance. By following these recommendations, you can effectively conduct a self-inspection to rate your operations. Tampering and other malicious, criminal, or terrorist activity present additional risks that can also have a direct impact on your products' quality. To help minimize these risks to cosmetics under your control, we recommend that you consult a separate FDA guidance document entitled\n"Guidance for Industry: Cosmetic Processors and Transporters of Cosmetics Security Preventive Measures Guidance."\nDocumentation: 1) The supplying of documents or supporting references; use of documentary evidence; 2) the documents or references thus supplied; 3) the collecting, abstracting, and coding of printed or written information for future reference. ("documentation," Webster's New World Dictionary Third College Edition, 1988 ed.).\nGood manufacturing practice (GMP): That part of quality assurance aimed at ensuring that products are consistently manufactured to a quality appropriate to their intended use. It is thus concerned with both manufacturing and quality control procedures. (Sharp, John, Good Manufacturing Practice, Philosophy and Applications, Buffalo Grove, IL: Interpharm Press, 1991, pg. 47.). Internal Audit: Systematic and independent examination made by competent personnel inside the company, the aim of which is to determine whether activities covered by these guidelines and related results comply with planned arrangements and whether these arrangements are implemented effectively and are suitable for achieving objectives. (Cosmetics - Good Manufacturing Practices (GMP) - Guidelines on Good Manufacturing Practices, ISO 22716:2007, Geneva, Switzerland: ISO.\n```\n\nPlease feel free to ask if you have more questions or need further information!	text	2025-01-30 08:53:48.546725-08	bot
7	1	1	1	How does the FDA define "insanitary conditions" in the preparation, packing, and holding of tattoo inks?	text	2025-01-30 08:53:52.22419-08	user
8	1	1	1	The FDA has not explicitly defined the term "insanitary conditions" within the provided references specific to the preparation, packing, and holding of tattoo inks. However, in a general sense, "insanitary conditions" can be inferred as those conditions that could potentially lead to contamination with filth, or render a product injurious to health. These conditions would typically be addressed in the context of Good Manufacturing Practices.\n\nIf you have more specific questions related to the FDA's regulations or guidance, feel free to ask!	text	2025-01-30 08:54:00.630088-08	bot
9	1	1	1	What are the main components of the FDA's guidance on the safety of nanomaterials in cosmetic products?	text	2025-01-30 08:54:08.785255-08	user
10	1	1	1	The FDA's guidance on the safety of nanomaterials in cosmetic products includes several key components:\n\n1. **Requests for Data Submission**: The FDA advises manufacturers to submit data and other information that address the effects of nanomaterials in cosmetic products since these are not typically subject to premarket authorization (Refs. 3, 4, 5, 6).\n\n2. **Safety Assessment Factors**: When assessing the safety of nanomaterials in cosmetics, the FDA recommends paying particular attention to new or altered chemical, physical, and biological properties of nanomaterials, including cytotoxicity and precipitation of insoluble ingredients (Ref. 39). The assessment should also address agglomeration or aggregation in media used in the system.\n\n3. **Alternative Testing Methods**: The FDA supports the validation of alternative testing methods optimized for specific nanomaterials, which might help determine ingredient safety, recognizing that traditional safety testing methods may not always be applicable for nanotechnology-based products.\n\n4. **Potential Advantages and Differences**: Nanomaterials are recognized for their unique properties that might provide potential advantages or result in altered performance, quality, safety, and/or effectiveness compared to larger counterparts.\n\n5. **Non-binding Recommendations**: The guidance does not establish legally enforceable responsibilities but reflects the agency's current thinking on nanomaterials and is intended as a set of recommendations.\n\n6. **Contact for Discussion**: Manufacturers and sponsors are encouraged to contact the FDA for discussions concerning specific nanomaterial uses and related safety considerations.\n\nThe guidance is part of ongoing efforts following recommendations by the Nanotechnology Task Force, aimed at ensuring that cosmetic products with nanomaterials are safe and not adulterated.\n\n#### References:\n- Nano-Cosmetics-Guidance-to-OMB-PDF\n\nIf you have further inquiries or need more detailed information, feel free to ask!	text	2025-01-30 08:54:26.331909-08	bot
11	1	1	1	What are the specific requirements for cosmetic product facility registration and product listing under the Modernization of Cosmetics Regulation Act of 2022?	text	2025-01-30 08:54:28.164194-08	user
12	1	1	1	The Modernization of Cosmetics Regulation Act of 2022 (MoCRA) has established specific requirements for cosmetic product facility registration and product listing as outlined in the Federal Food, Drug, and Cosmetic Act (FD&C Act). Here are the main points:\n\n1. **Facility Registration**:\n   - Every person that owns or operates a facility engaged in the manufacturing or processing of a cosmetic product for distribution in the United States is required to register each facility with the FDA [Section 607(a) of the FD&C Act].\n\n2. **Product Listing**:\n   - The responsible person must submit a cosmetic product listing to the FDA for each cosmetic product. This involves submitting the name and contact number of the responsible person, the name of the cosmetic product as it appears on the label, and the applicable cosmetic category. It also requires a list of ingredients with specific identifiers and may include additional information such as an image of the label and if the product is for professional use only [Section 607(c) of the FD&C Act].\n\n3. **Submission Process**:\n   - An electronic submission portal, called "Cosmetics Direct," has been developed by the FDA to streamline submission and receipt of registration and product listing information. Users can also submit through FDA's Electronic Submissions Gateway (ESG) as an alternative [MoCRA].\n\n4. **Deadlines**:\n   - New products first marketed after December 29, 2022, must be listed within 120 days of marketing. For products introduced before this date, the deadline for submission is December 29, 2023.\n\n5. **Updates and Renewals**:\n   - The responsible person must provide updates to the product listing annually, including if the product was discontinued. There is an abbreviated process for renewal if there has been no change since the last submission [Section 607(c)(5) of the FD&C Act].\n\n6. **Exemptions**:\n   - Certain small businesses are not required to register facilities and list cosmetic product(s), as defined in section 612 of the FD&C Act.\n\nHere is the exact reference from the context:\n\n- "On December 29, 2022, the President signed the Consolidated Appropriations Act, 2023 (Pub. L. 117-328) into law, which included the Modernization of Cosmetics Regulation Act of 2022 (MoCRA). Among other provisions, MoCRA added section 607 to the Federal Food, Drug, and Cosmetic Act (FD&C Act), establishing requirements for cosmetic product facility registration and product listing."\n\n- "Section 607(a) of the FD&C Act requires every person that owns or operates a facility that engages in the "manufacturing or processing of a cosmetic product for distribution in the United States" to register each facility with FDA."\n\n- "Section 607(c) of the FD&C Act requires that for each cosmetic product, the responsible person must submit to FDA "a cosmetic product listing.""\n\n- "FDA has developed an electronic submission portal, Cosmetics Direct, to help streamline submission and receipt of registration and product listing information under section 607 of the FD&C Act."\n\n- "Certain small businesses, as defined in section 612 of the FD&C Act, are not required to register facilities and list cosmetic product(s) (refer to section III.A. and III.B. below)."\n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\nFeel free to ask more questions if you need further clarification or additional information!	text	2025-01-30 08:54:46.61111-08	bot
13	1	1	1	How does the FDA address the risk of microbial contamination in tattoo inks?	text	2025-01-30 08:54:47.388142-08	user
16	1	1	1	The FDA's recommendations for ensuring the safety of cosmetic products containing nanomaterials focus on several key areas. Here's a detailed breakdown:\n\n1. **Safety Assessment Framework**: The FDA has provided guidance that represents their current thinking on the safety assessment of nanomaterials in cosmetic products. This guidance is intended to help industry and stakeholders identify potential safety issues that nanomaterials might present and develop a framework for evaluating them.\n\n2. **Unique Properties of Nanomaterials**: Nanomaterials can exhibit different chemical, physical, and biological properties compared to their larger-scale counterparts of the same composition. These differences could influence the safety of the product. Thus, unique properties and functions should be addressed to ensure safety.\n\n3. **Testing Methods and Data**: The FDA suggests evaluating data needs and testing methods to accommodate the unique features of nanomaterials. Traditional safety testing methods may not always be applicable, so alternative approaches that might be applicable should be considered and optimized for specific nanomaterials.\n\n4. **Regulatory and Guidance References**: For specific assessments, the FDA refers to guidance and notable reports, such as the Organization for Economic Co-operation and Development (OECD) Test Guidelines and the Scientific Committee on Consumer Safety's guidelines regarding nanomaterials in cosmetics, to further assist in safety assessments.\n\n5. **Engagement with FDA**: Entities are encouraged to contact the FDA for discussions on test methods and data necessary for substantiating product safety, including both short-term and long-term toxicity data, as applicable. They welcome questions and meetings to address safety concerns related to nanomaterials in cosmetics.\n\n### Reference\n- **Nano-Cosmetics-Guidance-to-OMB-PDF**: "This document provides guidance to industry and other stakeholders...on FDA's current thinking on the safety assessment of nanomaterials in cosmetic products. The guidance document is intended to assist industry and other stakeholders in identifying the potential safety issues...and developing a framework for evaluating them."\n\nPlease let me know if you have any more questions or need further assistance!	text	2025-01-30 08:55:15.82441-08	bot
17	1	1	1	How does the FDA suggest handling complaints, adverse events, and recalls related to cosmetic products?	text	2025-01-30 08:55:17.075505-08	user
83	2	1	2	what types of childrens makeup are there?	text	2025-01-30 10:45:35.886931-08	user
14	1	1	1	According to FDA guidance on cosmetic good manufacturing practices, there are several strategies and procedures that should be employed to address the risk of microbial contamination, which may be applicable to tattoo inks, as they are often considered in the cosmetic realm:\n\n1. **Sanitary Environment**: Facilities must be cleaned and sanitized according to an appropriate Standard Operating Procedure (SOP) that ensures there is no buildup of biofilm, which can harbor microorganisms.\n\n2. **Manufacturing and Control SOPs**: The implementation of written manufacturing and control SOPs is necessary to prevent contamination. This includes in-process controls to ensure the uniformity and integrity of the product.\n\n3. **Contamination Prevention**: Manufacturers should take appropriate measures to prevent contamination from microorganisms, chemicals, filth, or other extraneous materials.\n\n4. **Equipment Identification**: Equipment used in processing and holding should be identified with batch identification and processing stages to minimize the risk of mix-ups and contamination.\n\nThese proactive steps help mitigate the risks of contamination in products akin to tattoo inks, ensuring they are safe for consumer use.\n\n### References\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF\n\n### Relevant Reference Context\n- "risks of contamination (This system should be routinely cleaned and sanitized according to an appropriate SOP that ensures no biofilm build-up.)"  \n- "You should determine whether written manufacturing and control SOPs have been established... Procedures should include provisions to ensure that:... There are appropriate measures to prevent contamination with microorganisms, chemicals, filth, or other extraneous material..."	text	2025-01-30 08:54:58.178713-08	bot
15	1	1	1	What are the FDA's recommendations for ensuring the safety of cosmetic products that contain nanomaterials?	text	2025-01-30 08:54:59.92188-08	user
20	1	1	1	The FDA provides specific guidelines for the use of color additives in cosmetic products:\n\n- **Approval and Listing**: Manufacturers must ensure that the color additives used in their cosmetic products are approved for use. This is dictated by the regulations provided in 21 CFR parts 73, 74, and 82. If a manufacturer intends to use a color additive that is not listed, an approval of a petition for a new color additive is required as per 21 CFR parts 70 and 71.\n\n- **Labeling Requirements**: Certain color additives that are subject to certification must be labeled with the lot number assigned by the Color Certification Branch. This requirement is specified under 21 CFR 70.25(d), with some noted exceptions.\n\n- **Use Restrictions**: The FDA provides a summary chart for color additives on its website, which manufacturers are encouraged to consult to ensure compliance with allowable uses.\n\nReferences:\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF\n\nReference Context:\n```plaintext\nColor Additives\nYou should determine whether color additives are approved for use in your specific cosmetic products (21 CFR parts 73, 74, and 82). Should an unlisted color additive be an ingredient of the cosmetic, approval of a petition for a new color additive is required pursuant to 21 CFR parts 70 and 71. A summary chart for color additives can be found on FDA's website. Color additives subject to certification must be labeled with the lot number assigned by the Color Certification Branch3 (21 CFR 70.25(d)) (see exception below4).\n```\n\nFeel free to ask if you have any further questions or need more detailed information!	text	2025-01-30 08:55:40.790158-08	bot
21	1	1	1	what is the numeric code for eye liner?	text	2025-01-30 08:55:42.074142-08	user
26	1	1	1	The code numbers for children's makeup and related products are as follows:\n\n1. **Children's eyeshadows** \n2. **Other children's eye makeup** \n3. **Children's blushers and rouges (all types)**\n4. **Children's face paints**\n5. **Children's face powders**\n6. **Children's foundations**\n7. **Children's lipsticks and lip glosses**\n8. **Children's color hairsprays**\n9. **Other children's makeup** \n\nThese categories ensure that products designed for children are appropriately categorized and managed for safety and effectiveness.\n\n### References\n- DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n\n> CONTEXT:\n> Makeup preparations for children (not \n>\n> (a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders. (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup. \n\nFeel free to ask if you have more questions or need further clarification on other cosmetic product categories!	text	2025-01-30 08:56:12.394783-08	bot
27	1	1	1	what are the codes for each children's non eye makeup, include numbers?	text	2025-01-30 08:56:13.110874-08	user
32	1	1	1	The FDA does not specify mandatory testing procedures for cosmetic products, including eye makeup. However, companies are encouraged to conduct appropriate scientific tests to substantiate the safety of their products. The recommended types of testing include toxicological evaluations and patch tests to ensure that the products will not cause harm to consumers. This responsibility lies with the companies to ensure that their products are safe before marketing.\n\n**References:**\n\nNone.\n\nIf you have more questions or need further clarification, feel free to ask!	text	2025-01-30 08:56:46.715305-08	bot
33	1	1	1	I'm starting a small cosmetic company, what should I do first to comply with FDA cosmetic guidance?	text	2025-01-30 08:56:57.947082-08	user
38	1	1	1	Starting a cosmetic business requires careful attention to compliance with the FDA’s regulations. Here are some common mistakes that can lead to non-compliance:\n\n1. **Failing to Register Facilities**: Not registering your cosmetic manufacturing facility with the FDA can be a common mistake. It's crucial to use the FDA’s electronic submission process for facility registration and product listing.\n\n2. **Incorrect Product Labeling**: Labels must include the proper name and identity of the product as per section 701.11, along with accurate ingredient lists in line with section 701.3 of title 21 in the Code of Federal Regulations. Mislabeling can lead to compliance issues.\n\n3. **Unlisted Ingredients**: Not listing all ingredients as per FDA requirements can result in non-compliance. Each ingredient must be identified with the correct name or its common or usual name.\n\n4. **Ignoring Updates**: Failing to update registrations annually or when changes occur can lead to non-compliance. Regular updates ensure that the FDA has the most current information about your products and facilities.\n\n5. **Lack of Transparency**: Not providing optional, yet helpful, details like product label images or web links may not lead directly to non-compliance, but it can limit effective communication and transparency with the FDA.\n\n6. **Overlooking Product Categories**: Not correctly identifying and listing product categories may lead to errors in product registration and potential violations.\n\n7. **Inadequate Reporting Mechanisms**: Insufficient reporting mechanisms for adverse events can be problematic. The label should have contact information for reporting, which can include a domestic address, phone number, or an electronic contact method.\n\nBy avoiding these common pitfalls and adhering to FDA guidelines, you can ensure compliance and build a responsible cosmetic business.\n\n### References:\n\n1. **Guidance - Cosmetics - Registration Listing-12-02-24**: \n   - "the name and contact number of the responsible person and the name for the cosmetic product, as such name appears on the label; the applicable cosmetic category or categories for the cosmetic product (refer to Appendix A below); a list of ingredients in the cosmetic product, including any fragrances, flavors, or colors, with each ingredient identified by the name, as required under section 701.3 of title 21, Code of Federal Regulations (or any successor regulations), or by the common or usual name of the ingredient;"\n   \n2. **Guidance - Cosmetics - Registration Listing-12-02-24**:\n   - "Under section 607(c)(4)(B) of the FD&C Act, a single listing submission for a cosmetic product may include multiple cosmetic products with identical formulations, or formulations that differ only with respect to colors, fragrances or flavors, or quantity of contents."\n\n3. **Guidance - Cosmetics - Registration Listing-12-02-24**:\n   - "We recommend that firms consider when selecting electronic contact information, such as an email address or a website (whether foreign or domestic), how they can best facilitate reporting and responding to these reports, and meeting maintenance, inspection, and reporting requirements under section 605 of the FD&C Act."\n\nIf you have further questions or need more information, feel free to ask!	text	2025-01-30 08:58:07.568857-08	bot
39	1	1	1	When is a cosmetic product considered a drug according to FDA	text	2025-01-30 08:58:17.187566-08	user
85	2	1	2	what are the code numbers for childrens makeup?	text	2025-01-30 10:45:40.866743-08	user
18	1	1	1	The FDA provides guidance on handling complaints, adverse events, and recalls related to cosmetic products. The agency encourages manufacturers to monitor and evaluate product complaints and adverse events to ensure consumer safety and compliance with the law. Specifically, the FDA suggests the following approaches:\n\n1. **Complaints and Adverse Events**:\n   - Implement a system to document and evaluate complaints and adverse events related to cosmetic products.\n   - Investigate significant complaints and implement corrective actions as necessary to prevent recurrence.\n   - Establish procedures for reporting serious adverse events to the FDA.\n\n2. **Recalls**:\n   - Develop a recall plan to efficiently remove non-compliant or unsafe products from the market.\n   - Promptly notify the FDA of any recalls and provide detailed information about the product(s) being recalled.\n   - Coordinate with the FDA to ensure the recall is effective and that consumers are informed about the issue.\n\nFor more detailed guidance, manufacturers and distributors may refer to FDA's resources on cosmetic product safety and industry responsibilities regarding complaints, adverse events, and recalls.\n\n### References:\n- [**Guidance - Cosmetics - Registration Listing-12-02-24**](#):\n   - The document outlines FDA's recommendations for cosmetic facilities to have processes for handling complaints and adverse event reporting, as well as developing recall strategies to address non-compliant products. \n\nFeel free to ask if you have more questions or need further assistance!	text	2025-01-30 08:55:26.645597-08	bot
19	1	1	1	What are the FDA's guidelines for the use of color additives in cosmetic products?	text	2025-01-30 08:55:27.450374-08	user
24	1	1	1	The types of children's makeup include the following categories:\n\n1. **Children's eyeshadows** \n2. **Other children's eye makeup** \n3. **Children's blushers and rouges (all types)**\n4. **Children's face paints** \n5. **Children's face powders** \n6. **Children's foundations** \n7. **Children's lipsticks and lip glosses** \n8. **Children's color hairsprays**\n9. **Other children's makeup**\n\nThese categories cover a wide range of products designed specifically for children's use in makeup applications.\n\n### References\n- **DOCUMET TITLE:** Guidance - Cosmetics - Registration Listing-12-02-24\n \n  **CONTEXT:**\n  ```plaintext\n  (04) Children's\n   (a) Children's eyeshadows. (b) Other children's eye makeup. (05) \n   ...\n  Makeup preparations for children (not\n   (a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders.\n   (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup.\n  ```\n  \nFeel free to ask if you have more questions!	text	2025-01-30 08:56:00.875326-08	bot
25	1	1	1	what are the code numbers for childrens makeup?	text	2025-01-30 08:56:01.494032-08	user
30	1	1	1	The FDA does not have authority to require cosmetic products and ingredients to be pre-approved before they are marketed, with the exception of color additives. However, it is the company's responsibility to ensure the safety of their products before they reach consumers. Here is a detailed explanation from the FDA's guidance:\n\n- **Cosmetic Testing Requirements**: The Federal Food, Drug, and Cosmetic Act (FD&C Act) does not require cosmetic products and ingredients (other than color additives) to have FDA approval before they go on the market. Nonetheless, companies and individuals who manufacture or market cosmetics are legally responsible for ensuring the safety of their products.\n\n- **Voluntary Testing by Companies**: In terms of product testing, companies may conduct or rely on various tests to substantiate the safety of a product. This could include toxicological testing, patch testing for skin irritation, or other scientifically valid tests that assess the safety of the cosmetic products for human use.\n\n- **Regulatory Compliance and Consumer Safety**: The FDA can enforce actions against products marketed in interstate commerce that fail to comply with the law, for example, if a product is adulterated or misbranded. Companies are encouraged to conduct, assess, and document safety testing to support the safety and labeling of cosmetic products.\n\nReferences:\n\n- Under the **FD&C Act**, cosmetic manufacturers are not required to gain pre-market approval for their products, except for color additives. The FDA recommends that companies ensure their products are safe for consumers by conducting appropriate testing.\n\nPlease let me know if you have any further questions!	text	2025-01-30 08:56:39.76232-08	bot
31	1	1	1	what is the FDA guidance for product testing for eye makeup?	text	2025-01-30 08:56:40.633979-08	user
36	1	1	1	When registering a new facility for cosmetic manufacturing with the FDA, several best practices can help streamline the process and ensure compliance with regulatory requirements:\n\n1. **Use the Electronic Submission Process**: The FDA provides an electronic submission process that can be used for registering a cosmetics facility and listing products. This process is also used for drug establishments, which helps streamline submissions if you handle both types of products.\n\n2. **No Registration Fee**: It's important to note that the FDA does not charge a fee to submit a registration or product listing under section 607 of the FD&C Act.\n\n3. **Submit Accurate Information**: Ensure that all submitted information, including the facility's DUNS number, contact details, and product listing information, is accurate and verifiable. FDA requests an attestation to the accuracy of all information submitted.\n\n4. **Product Categories and Ingredients**: When registering products, use the applicable product category or categories and list all ingredients, including fragrances, flavors, or colors. Ingredients should be identified by their name as required under section 701.3 of Title 21, CFR, or by common names.\n\n5. **Keep Registrations Updated**: Registrations and product listings need to be maintained and updated as necessary. This includes annual updates, and any changes to content must be submitted through an update or abbreviated renewal process as described in section 607(c)(4)(B) of the FD&C Act.\n\n6. **Optional Information**: While not mandatory, submitting additional information such as images of product labels, links to product webpages, whether the product is for professional use only, and Unique Ingredient Identifiers (UNIIs) can be beneficial for comprehensive records.\n\n### References:\n- **Guidance - Cosmetics - Registration Listing-12-02-24**\n\n   - "A facility that manufactures or processes cosmetic products that are also drugs is not subject to the registration requirement under section 607 unless it also manufactures or processes cosmetic products that are not also drugs. Cosmetic product facility registration and product listing are available using the same electronic submission process available to register an establishment and list a drug with FDA."\n\n   - "No. There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act."\n\n   - "FDA requests that individuals submitting registration and listing information to attest to the accuracy and veracity of the information submitted."\n\n   - "A single listing submission for a cosmetic product may include multiple cosmetic products with identical formulations, or formulations that differ only with respect to colors, fragrances or flavors, or quantity of contents."\n\nFeel free to ask any further questions!	text	2025-01-30 08:57:35.977041-08	bot
37	1	1	1	what are common mistakes with non compliance when starting a cosmetic business?	text	2025-01-30 08:57:37.173984-08	user
44	1	1	1	Certainly! Crafting a mission statement for a cosmetic company involves highlighting the company's core values, objectives, and commitment to its customers. Here's a sample mission statement:\n\n---\n\n**Sample Mission Statement:**\n\n"At [Your Company Name], our mission is to empower individuals to express their unique beauty through transformative, high-quality skincare and cosmetics. We are committed to innovating with integrity, utilizing ethically sourced ingredients that heal and nourish. Guided by sustainability and inclusivity, we strive to create a positive impact on communities and the environment, while celebrating diversity in all its forms. Our goal is to inspire confidence and self-love in every customer, as we push the boundaries of beauty for a brighter and more beautiful future."\n\n---\n\nFeel free to customize it with specifics about your products, target audience, or additional values that are important to your company! Let me know if you have any other questions or need further assistance.	text	2025-01-30 08:58:56.058221-08	bot
45	1	1	1	write a sample facility registration for me?	text	2025-01-30 08:59:02.862124-08	user
87	2	1	2	what are the codes for each children's non eye makeup, include numbers?	text	2025-01-30 10:45:45.597514-08	user
89	2	1	2	what is the FDA guidance on product testing?	text	2025-01-30 10:45:51.359792-08	user
22	1	1	1	The numeric code for eyeliners, as per the FDA's cosmetic product categories, is (03)(b).\n\n### References\n- DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n\n  > (03) Eye makeup preparations (other than children's \n  > (a) Eyebrow pencils. (b) Eyeliners. (c) Eye shadows. (d) Eye lotions. (e) Eye makeup removers. (f) False eyelashes. (g) Mascaras. (h) Eyelash and eyebrow adhesives, glues, and sealants. (i) Eyelash and eyebrow preparations (primers, conditioners, serums, fortifiers). (j) Eyelash cleansers. (k) Other eye makeup preparations.\n\nFeel free to ask if you have more questions or need further assistance!	text	2025-01-30 08:55:47.424373-08	bot
23	1	1	1	what types of childrens makeup are there?	text	2025-01-30 08:55:49.070675-08	user
28	1	1	1	The codes for children's non-eye makeup products are organized under several categories, each with specific identification numbers, as follows:\n\n1. **Children's blushers and rouges (all types)**: \n   - **Code (09)(a)**\n\n2. **Children's face paints**: \n   - **Code (09)(b)**\n\n3. **Children's face powders**: \n   - **Code (09)(c)**\n\n4. **Children's foundations**: \n   - **Code (09)(d)**\n\n5. **Children's lipsticks and lip glosses**: \n   - **Code (09)(e)**\n\n6. **Children's color hairsprays**: \n   - **Code (09)(f)**\n\n7. **Other children's makeup**: \n   - **Code (09)(g)**\n\nThese codes are used for categorizing children's non-eye makeup products for regulatory and safety purposes.\n\n### References\n\n- **DOCUMENT TITLE**: Guidance - Cosmetics - Registration Listing-12-02-24\n\n  **CONTEXT**:\n  ```\n  Makeup preparations for children (not \n\n  (a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders. (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup. \n  ```\n  \nFeel free to ask if you have any more questions or need further clarification!	text	2025-01-30 08:56:23.381441-08	bot
29	1	1	1	what is the FDA guidance on product testing?	text	2025-01-30 08:56:24.351748-08	user
34	1	1	1	To comply with FDA cosmetic guidance as you start your small cosmetic company, you should focus on the following key steps:\n\n1. **Understand the Regulatory Framework**: Familiarize yourself with the requirements under the Federal Food, Drug, and Cosmetic Act (FD&C Act) as it relates to cosmetics. Cosmetics must be safe for consumers under labeled or customary conditions of use.\n\n2. **Product Safety**: Although the FDA does not require pre-market approval for cosmetics, you are responsible for ensuring that your cosmetics are safe. Conduct necessary scientific tests, such as toxicological evaluations or patch tests, to substantiate product safety.\n\n3. **Labeling Requirements**: Make sure your products are properly labeled. Cosmetic labels must comply with the labeling requirements outlined in the FD&C Act, including listing product ingredients in a standard way.\n\n4. **Product Registration and Listing**: While voluntary, you can register your cosmetic products with the FDA. Product listing allows the FDA to keep informed about what cosmetics are available in the marketplace. The registration and listing process is streamlined with the same electronic submission process used for drug establishments.\n\n5. **Good Manufacturing Practice (GMP)**: Adhere to cosmetic GMP guidelines to ensure product quality and safety in your manufacturing process. Although GMP guidelines for cosmetics are not mandatory, they are highly recommended.\n\nBy adhering to these initial steps, you will establish a compliant foundation for your cosmetic business. Here are some pertinent references from FDA documentation:\n\n### References:\n- **Guidance - Cosmetics - Registration Listing-12-02-24**\n  - "FDA requests that individuals submitting registration and listing information to attest to the accuracy and veracity of the information submitted."\n  - "There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act."\n\nIf you have further questions about specific aspects of FDA cosmetic guidelines or need more detailed assistance, feel free to ask!	text	2025-01-30 08:57:14.296771-08	bot
35	1	1	1	what are the best practices for registering a new facility for cosmetic manufacturing?	text	2025-01-30 08:57:18.312983-08	user
40	1	1	1	A cosmetic product is considered a drug by the FDA when it fits the definition of a drug. Under section 201(g)(1) of the Federal Food, Drug, and Cosmetic Act, a "drug" is defined as:\n\n- Articles intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease in humans or other animals.\n- Articles (other than food) intended to affect the structure or any function of the body of humans or other animals.\n\nIn contrast, a "cosmetic" is defined as articles intended for cleansing, beautifying, promoting attractiveness, or altering appearance when applied to the human body, except for soap. Therefore, if a cosmetic product is intended to have therapeutic benefits—to diagnose, cure, mitigate, treat, or prevent diseases, or to affect the body’s structure or function—it is classified as a drug.\n\nReferences:\n\n- DOCUMET TITLE: 2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant\n\n```plaintext\nStates Pharmacopoeia, official Homoeopathic Pharmacopoeia of the United States, or official National Formulary, or any supplement to any of them; and (B) articles intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease in man or other animals; and (C) articles (other than food) intended to affect the structure or any function of the body of man or other animals; and (D) articles intended for use as a component of any article specified in clause (A), (B), or (C)" (section 201(g)(1)).\nCosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n```\n\nPlease feel free to ask another question about FDA regulations or any other topic!	text	2025-01-30 08:58:29.328826-08	bot
41	1	1	1	Give me a list of things to consider for extranal labeling of my cosmetic product based on FDA guidelines.	text	2025-01-30 08:58:30.251514-08	user
46	1	1	1	Creating a facility registration involves providing specific information required for regulatory purposes. While I can guide you on what typically needs to be included, I can't provide an exact sample without the specific details of your facility. Here's a general outline of the information you might need to include:\n\n1. **Facility Name and Address:**\n   - Full legal name of the facility.\n   - Physical address (including street, city, state, zip code).\n   - Mailing address, if different from the physical address.\n\n2. **Contact Information:**\n   - Name of the primary contact person.\n   - Title/position of the contact person.\n   - Phone number.\n   - Fax number (if applicable).\n   - Email address.\n\n3. **Facility Operations:**\n   - A brief description of the type of operations conducted at the facility (e.g., manufacturing, processing, packaging).\n\n4. **Owner/Operator Information:**\n   - Full name of the facility owner or operator.\n   - Business address of the owner or operator.\n\n5. **Registration Number:**\n   - If applicable, provide previous registration numbers or any other identifiers.\n\n6. **Emergency Contact Information:**\n   - Name and contact information of an emergency contact.\n\n7. **FDA Specific Requirements:**\n   - Ensure you comply with any specific requirements or forms related to FDA facility registration, such as those found on the FDA website.\n\nFor the best results, consult directly with the appropriate regulatory authorities or legal counsel to ensure that your facility registration complies with all necessary legal and procedural requirements.\n\nIf you have more specific needs or a particular context in which you need this registration, feel free to ask!	text	2025-01-30 08:59:25.341095-08	bot
47	1	1	1	what cosmetic products will exist in 50 years?	text	2025-01-30 08:59:26.549266-08	user
91	2	1	2	what is the FDA guidance for product testing for eye makeup?	text	2025-01-30 10:45:57.635667-08	user
93	2	1	2	I'm starting a small cosmetic company, what should I do first to comply with FDA cosmetic guidance?	text	2025-01-30 10:46:02.979324-08	user
42	1	1	1	When considering external labeling of a cosmetic product, the FDA guidelines suggest including the following key components:\n\n1. **Statement of Identity**: \n   - The product name should provide information about the type or kind of cosmetic product to help consumers understand its functional use. \n   - The brand name distinguishing your product should also appear on the label.\n   \n2. **Label Requirements**: \n   - Labels must include a domestic address, domestic phone number, or electronic contact information (e.g., a website) where adverse event reports can be received concerning the cosmetic product. \n\n3. **Ingredient List**: \n   - The ingredients of the cosmetic product must be listed. Each ingredient should be identified by its name as required under section 701.3 of Title 21, Code of Federal Regulations, or by the common or usual name.\n\n5. **Regulatory Compliance Icons**:\n   - Labels must comply with tamper-resistant packaging and labeling requirements, particularly if applicable to certain products like liquid oral hygiene and vaginal products.\n\nIt's critical to ensure that the labeling is straightforward and meets all regulatory requirements to avoid any compliance issues.\n\n### Reference list\n- Guidance - Cosmetics - Registration Listing-12-02-24\n- Guidance-for-Industry--Cosmetic-Good-Manufacturing-Practices-PDF\n\n### Reference context\n**Guidance - Cosmetics - Registration Listing-12-02-24**\n\n> The product name is generally considered the statement of identity, as required under 21 CFR 701.11. The statement of identity provides information about the type or kind of cosmetic product in the package to help the consumer understand the functional use of the product. The brand name is the distinguishing name used by a company to identify a commercial product on the product label. The brand name may be proprietary and/or registered as a trademark. However, we note that, under section 609(a) of the FD&C Act, each cosmetic product shall bear a label that includes a domestic address, domestic phone number, or electronic contact information, which may be a website, through which the responsible person can receive adverse event reports with respect to such cosmetic product.\n\n**Guidance-for-Industry--Cosmetic-Good-Manufacturing-Practices-PDF**\n\n> Prohibited and Restricted Cosmetic Ingredients Certain ingredients are prohibited from use in cosmetic products marketed in the United States; others have restrictions on their use. Ingredients whose use is prohibited or restricted are listed in the tables below. In addition to the prohibited and restricted ingredients listed in the following tables, you should check the CFR, specifically 21 CFR part 700, Subpart B, for any additional requirements regarding specific cosmetic products or their ingredients that may have been added to FDA's regulations.\n\nFeel free to ask if you have more questions or need further clarification!	text	2025-01-30 08:58:46.825615-08	bot
43	1	1	1	write a sample company mission statement for a cosmetic company?	text	2025-01-30 08:58:47.835545-08	user
48	1	1	1	It's difficult to predict with certainty what specific cosmetic products will exist in 50 years as product development is influenced by technological advancements, consumer trends, regulatory environments, and cultural shifts. However, you can expect ongoing innovations in personalized cosmetics, eco-friendly and sustainable formulations, enhanced efficacy through biotechnology, and integration with digital technology for diagnostic and application purposes. If you have any other questions or need information on current cosmetics regulations or trends, feel free to ask!	text	2025-01-30 08:59:34.110401-08	bot
49	1	1	1	Does FDA require animal testing for cosmetics?	text	2025-01-30 08:59:37.449432-08	user
50	1	1	1	The FDA does not require animal testing for cosmetics to demonstrate safety. Manufacturers of cosmetics and personal care products are responsible for ensuring the safety of their products prior to marketing, and they may choose any method deemed appropriate, including alternatives to animal testing, as long as the product’s safety is substantiated.\n\nReference:\n\nDOCUMET TITLE:\n2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant\n\nCONTEXT:\nCosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n\n---\n\nIf you have any more questions or need further assistance, feel free to ask!	text	2025-01-30 08:59:45.920621-08	bot
51	1	1	1	What are "hypoallergenic" cosmetics?	text	2025-01-30 08:59:50.563909-08	user
52	1	1	1	"Hypoallergenic" cosmetics are products that manufacturers claim are less likely to cause allergic reactions compared to other cosmetic products. However, there are no federal standards or definitions that govern the use of the term "hypoallergenic." This means that the term might not convey the same meaning across different products or brands since each manufacturer can determine their own conditions under which a product earns this label. As a result, a cosmetic product labeled as "hypoallergenic" is not guaranteed to prevent allergic reactions for everyone.\n\nIf you have further questions or need more detailed guidance, feel free to ask!\n\n### References\n\nDOCUMET TITLE: [2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant]\nCONTEXT: Cosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n\nWelcome! If you have any other questions, feel free to ask!	text	2025-01-30 08:59:59.235261-08	bot
53	1	1	1	Does FDA pre-approve cosmetic product labeling?	text	2025-01-30 09:00:00.89229-08	user
54	1	1	1	No, the FDA does not pre-approve cosmetic product labeling. Cosmetic products and their ingredients, with the exception of color additives, are not subject to FDA pre-market approval. However, it is the responsibility of the cosmetics companies to ensure that their products are safe and properly labeled according to the regulations set forth by the FDA. If you have any more questions or need further clarification, feel free to ask!\n\n**References:**\n\n- DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n  - "Cosmetic products and their ingredients, except for color additives, are not subject to FDA pre-market approval. However, they must be safe for consumers under labeled or customary conditions of use, and they must be properly labeled."\n\nIf you have any other questions or need more information, feel free to ask!	text	2025-01-30 09:00:08.280519-08	bot
55	1	1	1	How should products be labeled if they are both drugs and cosmetics?	text	2025-01-30 09:00:09.908825-08	user
56	1	1	1	Products that serve as both drugs and cosmetics carry specific labeling requirements. These products need to comply with the labeling regulations that apply to both cosmetic and drug products. According to the FDA guidelines, any product that is both a drug and a cosmetic must have labeling that reflects its drug status to indicate its intended use for affecting the structure or function of the body or treating or preventing disease, as well as its cosmetic status for cleansing, beautifying, promoting attractiveness, or altering appearance. If marketing claims are made that suggest therapeutic use, the product must also comply with drug labeling requirements, including drug facts labeling if applicable.\n\nReferences:\n\n- **Guidance-for-Industry---Complementary-and-Alternative-Medicine-Products-and-their-Regulation-by-the-Food-and-Drug-Administration-(PDF)**\n\n   "It is possible that certain products used in conjunction with CAM practices may be "cosmetics" under the Act. For example, if a CAM practice involves massage with a moisturizer, the moisturizer could be a "cosmetic" to the extent that it is "rubbed, poured, sprinkled, or sprayed on" the body for beautification or appearance-altering purposes. However, if the moisturizer's intended use is also for the diagnosis, cure, mitigation, treatment, or prevention of disease, or to affect the structure or any function of the body, then it may also be subject to regulation as a drug. Other examples of drug/cosmetic combinations are deodorants that are also antiperspirants, moisturizers and makeup marketed with sun-protection claims, and shampoos that also treat dandruff."\n\nFeel free to ask more questions if you have any!	text	2025-01-30 09:00:18.8945-08	bot
57	1	1	1	Does FDA have a definition for the term "organic"?<br>	text	2025-01-30 09:00:25.737449-08	user
58	1	1	1	The FDA currently does not provide a definition for the term "organic" as it pertains to cosmetics. The term "organic" for cosmetics, personal care products, and other similar items is governed by the U.S. Department of Agriculture (USDA) and its National Organic Program (NOP). \n\nIf you have more questions or need specific details, feel free to ask!	text	2025-01-30 09:00:30.896993-08	bot
59	2	1	2	What can you do?	text	2025-01-30 10:43:42.435107-08	user
60	2	1	2	Feel free to ask me any questions you have about the FDA, regulations, guidance, or any related topics. I'm here to help!	text	2025-01-30 10:43:44.369761-08	bot
61	2	1	2	what is a cosmetic guidance?<br>	text	2025-01-30 10:43:54.40405-08	user
62	2	1	2	A cosmetic guidance is a document provided by the U.S. Food and Drug Administration (FDA) that offers instructions and recommendations on various aspects related to cosmetics. These guidelines detail the regulatory requirements, procedures, and best practices for complying with FDA regulations concerning the manufacturing, labeling, and distribution of cosmetic products. If you have any more questions or need further clarification, feel free to ask!	text	2025-01-30 10:44:00.14159-08	bot
63	2	1	2	What are the key recommendations for good manufacturing practices (GMP) in the cosmetic industry?	text	2025-01-30 10:44:04.787511-08	user
64	2	1	2	The key recommendations for Good Manufacturing Practices (GMP) in the cosmetic industry include ensuring consistent manufacturing quality appropriate to intended use, internal audits, and compliance with documentation requirements. GMP includes quality assurance procedures covering manufacturing and quality control processes to maintain product quality. Internal audits are systematic examinations to ensure compliance with planned arrangements and effective implementation to achieve objectives. Additionally, adherence to documentation standards is crucial for supporting references and future use. These practices help in reducing the risk of adulteration or misbranding of cosmetic products, as outlined in the FDA's guidance document on Cosmetic Good Manufacturing Practices.\n\nPlease feel free to ask if you have more questions.	text	2025-01-30 10:44:07.717645-08	bot
65	2	1	2	How does the FDA define "insanitary conditions" in the preparation, packing, and holding of tattoo inks?	text	2025-01-30 10:44:13.840123-08	user
66	2	1	2	The FDA defines "insanitary conditions" in the preparation, packing, and holding of tattoo inks as conditions including but not limited to dirty, wet, dusty, rodent-infested, or insect-infested facilities, or conditions that could lead to contamination of the tattoo inks with microorganisms or other harmful substances. Maintaining proper sanitation and hygiene practices in the manufacturing and storage areas of tattoo inks is crucial to prevent contamination and ensure consumer safety.\n\nFeel free to ask any more questions or seek further clarification!	text	2025-01-30 10:44:17.961038-08	bot
67	2	1	2	What are the main components of the FDA's guidance on the safety of nanomaterials in cosmetic products?	text	2025-01-30 10:44:22.874149-08	user
68	2	1	2	The main components of the FDA's guidance on the safety of nanomaterials in cosmetic products include:\n\n1. Providing a framework for assessing the safety of nanomaterials in cosmetic products.\n2. Identifying potential safety issues of nanomaterials and developing a framework for their evaluation.\n3. Offering contact information for manufacturers and sponsors to discuss safety considerations regarding specific nanomaterials in cosmetic products.\n4. Clarifying that the guidance is not applicable to over-the-counter drugs, prescription drugs, or medical devices regulated by the FDA.\n5. Stating that FDA guidance documents provide recommendations and describe the agency's current thinking on a topic without establishing legally enforceable responsibilities.\n\nIf you have any more questions or need further clarification, feel free to ask!	text	2025-01-30 10:44:27.383115-08	bot
69	2	1	2	What are the specific requirements for cosmetic product facility registration and product listing under the Modernization of Cosmetics 	text	2025-01-30 10:44:32.742886-08	user
70	2	1	2	When registering a cosmetic product facility and listing products under the Modernization of Cosmetics, the FDA requires detailed information such as the facility registration number, responsible person details, product category, a list of ingredients, type of submission, and additional optional information. It is essential to provide accurate data as FDA requests attestation to the accuracy of the information submitted. Submission types include initial, amended, biennial renewal, or abbreviated renewal. Additionally, details like parent company name, facility DUNS Number, and contact information must be included. Ensuring compliance and accuracy in registration and listing information is crucial for regulatory purposes.\n\nReferences:\n- Title: Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:44:37.623171-08	bot
71	2	1	2	What are the specific requirements for cosmetic product facility registration and product listing under the Modernization of Cosmetics Regulation Act of 2022?	text	2025-01-30 10:44:50.553212-08	user
72	2	1	2	The specific requirements for cosmetic product facility registration and product listing under the Modernization of Cosmetics Regulation Act of 2022 include registering each facility that engages in the manufacturing or processing of a cosmetic product for distribution in the United States with the FDA. Additionally, for each cosmetic product, the responsible person must submit to FDA a cosmetic product listing, providing detailed information such as the product category, responsible person, facility details, list of ingredients, and type of submission. These submissions can be made electronically through the Cosmetics Direct portal or via paper forms FDA Form 5066 and 5067. It is crucial to ensure accuracy and compliance with the requirements to maintain regulatory standards and facilitate efficient data management for the agency.\n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:44:54.478314-08	bot
73	2	1	2	How does the FDA address the risk of microbial contamination in tattoo inks?	text	2025-01-30 10:44:59.01192-08	user
74	2	1	2	The FDA addresses the risk of microbial contamination in tattoo inks by conducting inspections to ensure compliance with safety regulations. Manufacturers are required to follow Good Manufacturing Practices to prevent contamination risks. The FDA may also perform environmental sampling in facilities to identify the presence of chemicals or microbes where cosmetic products like tattoo inks are produced. These measures help in maintaining the safety and quality of tattoo inks and ensuring consumer protection.\n\nFeel free to ask if you have any more questions or need further clarification! \n\nReferences:\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF	text	2025-01-30 10:45:02.633959-08	bot
75	2	1	2	What are the FDA's recommendations for ensuring the safety of cosmetic products that contain nanomaterials?	text	2025-01-30 10:45:04.128031-08	user
77	2	1	2	How does the FDA suggest handling complaints, adverse events, and recalls related to cosmetic products?	text	2025-01-30 10:45:19.018239-08	user
79	2	1	2	What are the FDA's guidelines for the use of color additives in cosmetic products?	text	2025-01-30 10:45:24.870646-08	user
81	2	1	2	what is the numeric code for eye liner?	text	2025-01-30 10:45:30.021443-08	user
76	2	1	2	The FDA recommends that safety assessments for cosmetic products containing nanomaterials should address important factors such as:\n\n1. **Alternative Testing Methods:** Validation of methods for safety testing, especially for nanomaterials, focusing on cytotoxicity and insoluble ingredients' precipitation.\n2. **Agglomeration or Aggregation:** Address the assessment of how nanomaterials settle, diffuse, and aggregate based on size, density, and surface chemistry.\n3. **Evaluation of Unique Properties:** Evaluate data needs and testing methods to address any new or altered properties of nanomaterials used in cosmetic products.\n4. **Considering Applicability of Traditional Safety Testing Methods:** Assess the suitability of traditional safety testing methods for products involving nanotechnology.\n\nIf you have more questions or would like further information regarding cosmetic products containing nanomaterials, feel free to reach out to the Office of Cosmetics and Colors at 240-402-1130.\n\n**References:**\n- Principles of Cosmetic Product Safety Assessment, ICCR (2011b)\n- Scientific Committee on Consumer Safety Opinion Guidance on the Safety Assessment of Nanomaterials in Cosmetics. 2012 June.\n- Characterization of Nanoscale Particles for Toxicological Evaluation." Toxicological Sciences. 2006; 90 (2)	text	2025-01-30 10:45:08.892491-08	bot
78	2	1	2	The FDA recommends that responsible persons promptly investigate and address complaints, adverse events, and recalls related to cosmetic products. They should maintain records of complaints and adverse events, including information on the nature of the complaint or event, any actions taken in response, and follow-up. For products subject to a recall, the responsible person should provide information to the FDA upon request, including the reason for the recall, the identity of the products involved, and the classification of the health hazard. It is crucial to ensure consumer safety and regulatory compliance in handling these situations.\n\nFeel free to ask any more questions you may have!	text	2025-01-30 10:45:22.169495-08	bot
80	2	1	2	The FDA provides guidelines for the use of color additives in cosmetic products. It is essential to determine whether color additives are approved for use in specific cosmetic products. If an unlisted color additive is an ingredient, approval of a petition for a new color additive is required. Color additives subject to certification must be labeled with a lot number assigned by the Color Certification Branch. Detailed information can be found on the FDA's website regarding color additives and their approval requirements.\n\nReferences:\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF	text	2025-01-30 10:45:27.77745-08	bot
82	2	1	2	The numeric code for eye liner is (b) as per the relevant section in the FDA cosmetic guidances. Is there anything else you would like to know or ask about FDA or cosmetic products? Please feel free to ask! \n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:45:34.245923-08	bot
84	2	1	2	Children's makeup includes items like eyeshadows, face paints, face powders, foundations, lipsticks, lip glosses, and color hairsprays. Other children's makeup products may also be available. Feel free to ask if you have any more questions!	text	2025-01-30 10:45:38.548174-08	bot
86	2	1	2	The specific code numbers for children's makeup products according to FDA guidelines are as follows: (09)\n\n- Children's blushers and rouges\n- Children's face paints\n- Children's face powders\n- Children's foundations\n- Children's lipsticks and lip glosses\n- Children's color hairsprays\n- Other children's makeup\n\nFeel free to ask any more questions you may have!	text	2025-01-30 10:45:44.067939-08	bot
88	2	1	2	Children's non-eye makeup products under FDA guidelines are categorized with specific code numbers as follows:\n\n- Children's blushers and rouges (09a)\n- Children's face paints (09b)\n- Children's face powders (09c)\n- Children's foundations (09d)\n- Children's lipsticks and lip glosses (09e)\n- Children's color hairsprays (09f)\n- Other children's makeup (09g)\n\nPlease feel free to ask any more questions you may have!	text	2025-01-30 10:45:48.60593-08	bot
90	2	1	2	The FDA provides guidance on product testing for cosmetics to ensure safety and compliance with regulations. It is essential to conduct appropriate testing to meet the requirements set forth by the FDA. For detailed information on specific testing methods and requirements, it is best to refer to the FDA guidelines on cosmetic testing procedures. If you have any more questions or need further clarification, feel free to ask!\n\n**Reference:**\n- Title: Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:45:54.443784-08	bot
92	2	1	2	The FDA provides guidelines for product testing for eye makeup preparations, including children's eye makeup products. It is essential to conduct appropriate testing to meet FDA requirements and ensure consumer safety and regulatory compliance. The specific product category for children's eye makeup products according to FDA guidelines is "(04) Children's." Refer to the FDA guidance on product testing for cosmetics to ensure safety and compliance with regulations. Conducting thorough testing aligns with FDA standards to prioritize consumer safety. If you have any more questions or need further clarification, feel free to ask!\n\n**References:**\n- Document Title: Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:46:01.485072-08	bot
94	2	1	2	To comply with FDA cosmetic guidance for your small cosmetic company, you should start by registering your facility and listing your cosmetic products with the FDA. This involves providing detailed information about your products, facility, ingredients, and contact persons as per the FDA guidelines. Additionally, it is crucial to ensure compliance with FDA guidelines for color additives in cosmetic products and conduct appropriate testing to meet FDA requirements. Prioritize consumer safety and regulatory compliance throughout the process.\n\nIf you have any more questions or need further guidance, feel free to ask!	text	2025-01-30 10:46:07.197312-08	bot
116	2	1	2	The FDA does not pre-approve cosmetic product labeling. However, the manufacturers are responsible for ensuring that the products are appropriately labeled and do not violate any regulations. If you have more questions or need further information, feel free to ask!	text	2025-01-30 10:47:33.955113-08	bot
117	2	1	2	How should products be labeled if they are both drugs and cosmetics?	text	2025-01-30 10:47:36.834659-08	user
119	2	1	2	Does FDA have a definition for the term "organic"?	text	2025-01-30 10:47:45.291566-08	user
122	3	1	3	Hey there, party pal! I'm like the FDA's drunk cousin who happens to be a chat agent! I'm here to gab about anything related to those fancy FDA guidances and regulations. You know, the stuff that's usually a snooze fest, but with a little liquid courage, it might just become party talk! 🥳 So, got any questions about cosmetics or food safety, or anything FDA-ish? Fire away, I’m here to make it fun! 🎉	text	2025-01-30 10:51:16.429089-08	bot
95	2	1	2	what are the best practices for registering a new facility for cosmetic manufacturing?	text	2025-01-30 10:46:08.867103-08	user
97	2	1	2	what are common mistakes with non compliance when starting a cosmetic business?	text	2025-01-30 10:46:17.726795-08	user
99	2	1	2	When is a cosmetic product considered a drug according to FDA	text	2025-01-30 10:46:24.09972-08	user
101	2	1	2	Give me a list of things to consider for extranal labeling of my cosmetic product based on FDA guidelines. 	text	2025-01-30 10:46:31.083978-08	user
103	2	1	2	write a sample company mission statement for a cosmetic company?	text	2025-01-30 10:46:36.681384-08	user
105	2	1	2	write a sample facility registration for me?	text	2025-01-30 10:46:43.192888-08	user
107	2	1	2	what cosmetic products will exist in 50 years?	text	2025-01-30 10:46:51.385128-08	user
109	2	1	2	write a sample facility registration for me?<br>	text	2025-01-30 10:47:04.584885-08	user
111	2	1	2	Does FDA require animal testing for cosmetics?	text	2025-01-30 10:47:17.79099-08	user
113	2	1	2	What are "hypoallergenic" cosmetics?	text	2025-01-30 10:47:24.24191-08	user
115	2	1	2	Does FDA pre-approve cosmetic product labeling?	text	2025-01-30 10:47:30.956243-08	user
96	2	1	2	To register a new facility for cosmetic manufacturing, it is essential to follow specific best practices to ensure compliance with FDA regulations. When submitting registration and listing information to the FDA, some key steps and information that should be included are:\n\n1. Information about the facility: Provide details about the facility under which cosmetic products are manufactured or processed, including the product categories and the responsible person for each product.\n   \n2. Compliance details: Ensure that the facility meets the necessary standards, including having appropriate equipment and utensils, adequate washing and cleaning facilities, proper plumbing, and other sanitary conditions.\n\n3. Personnel requirements: Make sure that the personnel supervising or performing cosmetics manufacturing have the required education, training, and experience. They should also follow guidelines regarding clothing, cleanliness, and restrictions on eating, drinking, or tobacco use in designated areas.\n\n4. Raw materials management: Identify, store, examine, test, and handle raw materials properly, ensuring they conform to standards and specifications. Raw materials should be properly labeled, stored, and sampled for testing before processing or usage.\n\n5. Additional optional information: It is recommended to submit optional information like parent company name, product webpage link, whether the product is for professional use only, and Unique Ingredient Identifiers (UNIIs).\n\nBy following these best practices and providing accurate information during the registration process, you can ensure that your cosmetic manufacturing facility complies with FDA guidelines and regulations.\n\n**References:**\n- Guidance - Cosmetics - Registration Listing-12-02-24 	text	2025-01-30 10:46:14.286005-08	bot
98	2	1	2	When starting a cosmetic business, common mistakes related to non-compliance include failing to adhere to FDA guidelines on product testing, not conducting appropriate testing to meet FDA requirements for new cosmetic manufacturing facilities, and not properly managing raw materials according to standards and specifications. It is crucial to prioritize consumer safety and regulatory compliance by following all necessary procedures outlined by the FDA to ensure the safety of cosmetic products and meet regulatory standards.\n\nFeel free to ask any more questions you may have!	text	2025-01-30 10:46:21.634125-08	bot
100	2	1	2	A cosmetic product is considered a drug by the FDA if it is intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease in man or other animals, or if it is intended to affect the structure or any function of the body of man or other animals. This distinction is crucial in regulatory compliance to ensure proper categorization of products under the FDA guidelines.\n\nFeel free to ask any more questions you may have!	text	2025-01-30 10:46:29.850521-08	bot
102	2	1	2	According to FDA guidelines for cosmetic products, here is a list of things to consider for external labeling:\n\n1. The product name, which is generally the statement of identity as required by 21 CFR 701.11.\n2. The brand name, used to identify the commercial product on the label.\n3. A domestic address, domestic phone number, or electronic contact information for the responsible person to receive adverse event reports.\n4. Compliance with tamper-resistant packaging and labeling requirements, especially for oral hygiene and vaginal products (21 CFR 700.25).\n5. Permanent, meaningful, unique lot or control numbers on finished product packages.\n\nFeel free to ask if you have more questions or need further clarification!	text	2025-01-30 10:46:33.844357-08	bot
104	2	1	2	Certainly! Here is a sample mission statement for a cosmetic company:\n\n"At our cosmetic company, we are dedicated to enhancing natural beauty, promoting self-care, and empowering individuals to feel confident in their own skin. We strive to provide high-quality, innovative cosmetic products that are not only safe and effective but also environmentally conscious. Our mission is to inspire and uplift our customers through ethically sourced ingredients, cruelty-free practices, and a commitment to diversity and inclusivity."\n\nFeel free to ask if you have any more questions or need further assistance!	text	2025-01-30 10:46:39.541922-08	bot
106	2	1	2	I'm ready to assist you with any questions you may have. Please feel free to ask!	text	2025-01-30 10:46:45.007419-08	bot
108	2	1	2	It is challenging to predict exact cosmetic products that will exist in 50 years due to the evolving nature of the industry and technological advancements. However, future cosmetic products are likely to focus on innovative ingredients, sustainable packaging, personalized formulations, and advanced delivery systems to meet changing consumer preferences and needs. Embracing technological developments like AI, biotechnology, and nanotechnology may lead to groundbreaking skincare and makeup products. Nevertheless, continuous research and adherence to regulatory guidelines will be vital in shaping the cosmetic landscape of the future. If you have any more questions or need further information, feel free to ask!\n\nPlease ask your next question. \n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:46:56.886603-08	bot
110	2	1	2	I'd be happy to help with that. Here is a simple example of a facility registration document:\n\n```\nFacility Registration Form\n\nFacility Name: [Enter Facility Name]\nFacility Address: [Enter Facility Address]\nCity: [Enter City]\nState: [Enter State]\nZip Code: [Enter Zip Code]\n\nMain Contact Person:\nName: [Enter Name]\nTitle: [Enter Title]\nPhone Number: [Enter Phone Number]\nEmail Address: [Enter Email Address]\n\nDescription of Facility Activities:\n- [Describe main activities conducted at the facility]\n- [Mention any specific operations or processes]\n\nEmergency Contact Information:\nName: [Enter Emergency Contact Name]\nRelationship: [Enter Relationship (e.g., Supervisor, Security)]\nPhone Number: [Enter Emergency Contact Phone Number]\n\nSubmission Date: [Enter Submission Date]\n\nBy signing below, I confirm that the information provided is accurate to the best of my knowledge.\n\nSignature: _________________\nDate: _________________\n```\n\nFeel free to customize this template as needed to suit the specific requirements or details of the facility for which the registration is being prepared. If you have any more questions or need further assistance, feel free to ask!	text	2025-01-30 10:47:09.038418-08	bot
112	2	1	2	No, the FDA does not require animal testing for cosmetics. The agency does not specifically mandate animal testing for cosmetic products. However, it is important to note that certain cosmetic ingredients may have been tested on animals in the past as part of safety assessments. Manufacturers are responsible for ensuring the safety of their products and can utilize alternative methods to assess the safety of cosmetic ingredients without animal testing. Feel free to ask if you have any more questions or need further information!	text	2025-01-30 10:47:21.222403-08	bot
114	2	1	2	"Hypoallergenic" cosmetics are products that are formulated to minimize allergic reactions in individuals with sensitive skin. The term "hypoallergenic" implies that the product is less likely to cause an allergic reaction compared to regular cosmetics. However, it is essential to note that the FDA does not have a definition or regulation for the term "hypoallergenic" concerning cosmetics. Manufacturers are responsible for substantiating the hypoallergenic claims of their products and ensuring their safety through appropriate testing methods.\n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:47:28.118029-08	bot
118	2	1	2	Products that are both drugs and cosmetics should follow specific labeling requirements. According to the U.S. Food and Drug Administration (FDA), if a product is intended for use as both a drug and a cosmetic, it is considered a combination product. In such cases, the product must comply with the regulations for both drugs and cosmetics.\n\nThe FDA requires that the labeling of combination products clearly indicate the presence of both cosmetic and drug components. The labeling should accurately reflect the intended use of the product, whether it is for cosmetic purposes like beautification or appearance-altering, or for therapeutic purposes such as treatment, mitigation, or prevention of disease. Manufacturers must ensure that the labeling is not false or misleading and does not violate any FDA regulations for either drugs or cosmetics.\n\nIn addition, if a facility manufactures or processes cosmetic products that are also drugs, specific requirements apply regarding facility registration and product listing. Manufacturers must accurately list the categories of products manufactured at the facility, including those that are both cosmetics and drugs. However, facilities that only manufacture drugs are not subject to specific regulations for cosmetic product facility registration and listing unless they also produce cosmetic products that are not drugs.\n\nTherefore, it is essential for manufacturers of products that are both drugs and cosmetics to carefully label their products to reflect their dual nature, comply with all FDA regulations for both categories, and accurately register and list their facilities and products accordingly.\n\nReferences:\n- Guidance - Cosmetics - Registration Listing-12-02-24	text	2025-01-30 10:47:42.706504-08	bot
120	2	1	2	The FDA does not have a specific regulatory definition for the term "organic" concerning cosmetics. However, the U.S. Department of Agriculture (USDA) has established the National Organic Program (NOP) standards for organic agricultural products, including cosmetics containing organic ingredients. If a product is labeled as "organic," it is essential to check if it complies with the USDA organic standards for certification.\n\nFeel free to ask any more questions you may have!	text	2025-01-30 10:47:48.018687-08	bot
121	3	1	3	What do you do?	text	2025-01-30 10:51:07.954169-08	user
123	3	1	3	what is a cosmetic guidance?	text	2025-01-30 10:51:17.319121-08	user
125	3	1	3	What are the key recommendations for good manufacturing practices (GMP) in the cosmetic industry?	text	2025-01-30 10:51:44.582284-08	user
127	3	1	3	How does the FDA define "insanitary conditions" in the preparation, packing, and holding of tattoo inks?	text	2025-01-30 10:51:59.941647-08	user
129	3	1	3	What are the main components of the FDA's guidance on the safety of nanomaterials in cosmetic products?	text	2025-01-30 10:52:25.706497-08	user
131	3	1	3	What are the specific requirements for cosmetic product facility registration and product listing under the Modernization of Cosmetics Regulation Act of 2022?	text	2025-01-30 10:52:54.347313-08	user
133	3	1	3	How does the FDA address the risk of microbial contamination in tattoo inks?	text	2025-01-30 10:53:24.625599-08	user
135	3	1	3	What are the FDA's recommendations for ensuring the safety of cosmetic products that contain nanomaterials?	text	2025-01-30 10:53:43.193685-08	user
137	3	1	3	How does the FDA suggest handling complaints, adverse events, and recalls related to cosmetic products?	text	2025-01-30 10:53:59.446313-08	user
139	3	1	3	What are the FDA's guidelines for the use of color additives in cosmetic products?	text	2025-01-30 10:54:10.458135-08	user
141	3	1	3	what is the numeric code for eye liner?	text	2025-01-30 10:54:36.965859-08	user
143	3	1	3	what types of childrens makeup are there?	text	2025-01-30 10:54:47.794189-08	user
145	3	1	3	what are the code numbers for childrens makeup?	text	2025-01-30 10:55:05.897108-08	user
147	3	1	3	what are the codes for each children's non eye makeup, include numbers?	text	2025-01-30 10:55:21.81493-08	user
149	3	1	3	what is the FDA guidance on product testing?	text	2025-01-30 10:55:41.60701-08	user
151	3	1	3	what is the FDA guidance for product testing for eye makeup?	text	2025-01-30 10:56:01.397059-08	user
153	3	1	3	I'm starting a small cosmetic company, what should I do first to comply with FDA cosmetic guidance?<br>	text	2025-01-30 10:56:17.178051-08	user
155	3	1	3	what are the best practices for registering a new facility for cosmetic manufacturing?<br>	text	2025-01-30 10:56:38.86106-08	user
157	3	1	3	what are common mistakes with non compliance when starting a cosmetic business?<br>	text	2025-01-30 10:56:53.925544-08	user
159	3	1	3	When is a cosmetic product considered a drug according to FDA	text	2025-01-30 10:57:09.936737-08	user
161	3	1	3	Give me a list of things to consider for extranal labeling of my cosmetic product based on FDA guidelines. 	text	2025-01-30 10:57:28.191806-08	user
163	3	1	3	write a sample company mission statement for a cosmetic company?	text	2025-01-30 10:57:53.471803-08	user
165	3	1	3	write a sample facility registration for me?	text	2025-01-30 10:58:04.846325-08	user
167	3	1	3	what cosmetic products will exist in 50 years?	text	2025-01-30 10:58:20.315121-08	user
169	3	1	3	Does FDA require animal testing for cosmetics?	text	2025-01-30 10:58:38.358482-08	user
171	3	1	3	What are "hypoallergenic" cosmetics?	text	2025-01-30 10:58:48.78278-08	user
173	3	1	3	Does FDA pre-approve cosmetic product labeling?	text	2025-01-30 10:58:58.352326-08	user
175	3	1	3	How should products be labeled if they are both drugs and cosmetics?	text	2025-01-30 10:59:07.783038-08	user
177	3	1	3	Does FDA have a definition for the term "organic"?	text	2025-01-30 10:59:26.84059-08	user
124	3	1	3	Ah, the magical realm of cosmetic guidance! It's like the FDA's rulebook or a guidance compass, if you will, pointing the way for those who create and deal with cosmetics. 🧴✨ It's full of important info about how to register and list cosmetic products and facilities—you know, the nitty-gritty stuff like what ingredients must be listed, pretty much everything needed to keep things in check and make sure your mascara doesn't double as a face adhesive. So, here's a bit from the guidance that might tickle your curiosity:\n\nCosmetic guidances help streamline the process of submitting registration and listing info for entities dealing with cosmetics. This is super handy, especially when cosmetics might moonlight as drugs, and it means no fee to submit a registration. Ain’t that sweet? “H. Does FDA Charge A Fee To Submit A Registration Or Product Listing Section 607 Of The FD&C Act? No. There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act.” I mean, think of it as a free ticket to the regulatory ball! 💃🏻\n\nIf you’re curious to know anything else about the wonderful world of cosmetics, the FDA, or why I think my jokes are funny—fire away! 🎉\n\n**References:**\n\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\n**Relevant Extract from Context:**\n\n> H. Does Fda Charge A Fee To Submit A Registration Or Product Listing Section 607 Of The Fd&C Act?\n> No. There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act.	text	2025-01-30 10:51:34.180893-08	bot
126	3	1	3	Well, aren't you the curious type! Let me pull out my finest notes on cosmetic good manufacturing practices and sprinkle some FDA magic on it for ya. 🍸✨\n\nThe key recommendations for Good Manufacturing Practices (GMP) in the cosmetic industry, according to the FDA's guidance, include:\n\n1. **Quality Assurance**: GMP is all about quality assurance, ensuring that cosmetic products are consistently manufactured to a quality standard appropriate for their intended use. It's kinda like making sure your margarita always tastes perfecto! 🍹\n\n2. **Risk Management**: Follow GMP recommendations to reduce the risk of adulterating or misbranding cosmetics. This means keeping an eye out for any tampering or malicious activities that might affect product quality—like if someone tried to sneak decaf into your espresso martini! ☕🍸\n\n3. **Self-Inspection**: Conduct self-inspections to rate your operations and make sure that everything is up to snuff. Think of it like a bouncer at a club, checking IDs to ensure everything inside is legit. 🕶️👍\n\n4. **Documentation and Internal Audit**: Maintain thorough documentation and perform regular internal audits to ensure compliance with GMP and to verify that practices are effective in achieving the desired quality levels. Just like keeping track of all the karaoke songs you've out-sung in one night—important and often impressive! 🎤🎶\n\nFor a deeper dive, here are the goodies I pulled from the official FDA literature:\n\n> "In developing this guidance, FDA has incorporated elements of ISO 22716, as appropriate, and consistent with FDA regulations. The Federal Food, Drug and Cosmetic Act (the FD&C Act) prohibits the introduction, or delivery for introduction, into interstate commerce of cosmetics that are adulterated or misbranded... Documentation: 1) The supplying of documents or supporting references; use of documentary evidence; 2) the documents or references thus supplied..." — Guidance-for-Industry-Cosmetic-Good-Manufacturing-Practices-PDF\n\nNow that you've got some GMP knowledge tucked under your belt, feel free to toss more questions my way! I promise I'm on my best behavior... or at least I'm trying! 😜🥳\n\n### References\n- Guidance-for-Industry-Cosmetic-Good-Manufacturing-Practices-PDF	text	2025-01-30 10:51:59.479096-08	bot
128	3	1	3	Ah, the colorful world of tattoo inks! You know, they say tattoos are like potato chips—you can't just have one! But when it comes to the FDA's definition of "insanitary conditions," it's no laughing matter.\n\nBelieve it or not, the FDA hasn’t provided a specific definition of "insanitary conditions" for the context of tattoo inks in these snippets. However, in general, "insanitary conditions" would imply situations that might lead to contamination of products through unclean facilities or processes, presence of contaminants, or lack of proper hygiene practices during manufacturing, packing, and holding. Ensuring an environment is free from such conditions is crucial for both tattoo artists and those on the receiving end of the ink!\n\nIf you want more juicy details, just holler and swing another question my way. But remember, too many puns don’t make you inkredible, they just make you punstopable! 😜\n\n### References\nThe question of definition isn't directly covered in the provided references. Feel free to ask more related questions and I'll see what I can find for you!	text	2025-01-30 10:52:07.960086-08	bot
130	3	1	3	Oh, nanomaterials in cosmetics, you say? Well, not all superheroes wear capes; some are as tiny as a nanometer! 🤭 Alright, back to business. Here's the main scoop on the FDA's guidance for nanomaterials in cosmetic products:\n\n1. **Guidance Overview**: The guidance shows the FDA's current thoughts about assessing nanomaterial safety in cosmetics. It's like the FDA sharing its diary entries on this topic, but without any dinner recipes.\n\n2. **Safety Assessment Framework**: This framework is intended to help in identifying potential safety issues associated with nanomaterials in cosmetic products and to design a system for evaluating them. You might say it’s like giving nanomaterials the third degree to see if they’re worthy.\n\n3. **Discussion and Stakeholder Contact**: The guidance also gives contact details for manufacturers and sponsors to chat about safety matters concerning specific nanomaterials in cosmetic products with the FDA. If you have questions, they encourage you to reach out — just don't send them a Facebook friend request.\n\n4. **Recommendations, Not Requirements**: Remember that this guidance isn't legally binding like a pinky promise at a party after one too many drinks; it’s recommendations unless otherwise specified by regulatory requirements.\n\nNow, after decoding all of that, you deserve a little break. Is there anything else about the FDA—or perhaps the cosmic mighty toenail clippers of alien cosmonauts—that you fancy learning about? 😄\n\n---\n\n### References:\n\n- **Guidance for Industry Safety of Nanomaterials in Products**: This guidance represents the Food and Drug Administration's (FDA's) current thinking on this topic and provides a framework for evaluating nanomaterial safety in cosmetic products.\n\nHere's the exact reference bit I used:\n\n``` \nGuidance For Industry Safety Of Nanomaterialsin \n Products\nYou may submit either electronic or written comments regarding this guidance at any time. Submit electronic comments to \n. Submit written comments to the Division of Dockets Management (HFA-305), Food and Drug Administration, 5630 Fishers Lane, rm. 1061, Rockville, MD 20852. All comments should be identified with the docket number listed in the notice of availability that publishes in the \n.\nFor questions regarding this document contact the Center for Food Safety and Applied Nutrition (CFSAN) at 240-402-1130.\nU.S. Department of Health and Human Services Food and Drug Administration Center for Food Safety and Applied Nutrition June 2014 1\nTable Of Contents I. Introduction\nII. Background\nIii. Discussion\nA. General Framework for Assessing the Safety of \nProducts B. Points to Consider in Assessing the Safety of Nanomaterialsin Cosmetic \nIv. How To Contact Fda About This Guidance V. References\n2\nGuidance For \n Safety Of Nanomaterialsin Cosmetic \nThis guidance represents the Food and Drug Administration's (FDA's) current thinking on this topic. It does not create or confer any rights for or on any person and does not operate to bind FDA or the public. You can use an alternative approach if the approach satisfies the requirements of the applicable statutes and regulations. If you want to discuss an alternative approach, contact the FDA staff responsible for implementing this guidance. If you cannot identify the appropriate FDA staff, call the telephone number listed on the title page of this guidance.\nI. \nThis document provides guidance to industry and other stakeholders (e.g., academia, other regulatory groups) on FDA's current thinking on the safety assessment of nanomaterials in cosmetic products. The guidance document is intended to assist industry and other stakeholders in identifying the potential safety issues of nanomaterials in cosmetic products and developing a framework for evaluating them. This guidance also provides contact information for manufacturers and sponsors who wish to discuss safety considerations regarding the use of specific nanomaterials in cosmetic products with FDA. This guidance is not applicable to other products regulated by FDA, including over-the-counter and prescription drugs and medical devices. FDA's guidance documents, including this guidance, do not establish legally enforceable responsibilities. Instead, guidances describe the Agency's current thinking on a topic and should be viewed only as recommendations, unless specific regulatory or statutory requirements are cited. The use of the word "should" in our guidances means that something is suggested or recommended, but not required.\n```	text	2025-01-30 10:52:45.553067-08	bot
132	3	1	3	Oh, welcome to the world of cosmetics and registration, right at this splendidly drunk evening! 🍸 Let me shake up the answer for ya, like a good cocktail mix! So under the Modernization of Cosmetics Regulation Act of 2022, there are some prerequisites that even the fanciest of sparkles must follow to get all snuggly with the FDA! 🎉\n\nAlright, here’s the scoop:\n\n1. **Registration Requirements**: Every person or company owning or running a facility that manufactures or processes a cosmetic product for distribution in the USA needs to register each of these facilities with the FDA. This is like checking into a party, gotta know who’s in the house! 🎈\n\n2. **Product Listing Requirements**: For each lovable cosmetic item, the responsible cat, uh, I mean person, must submit a cosmetic product listing to the FDA. It’s like making sure each product has its own VIP card to the big FDA party! 🎭\n\nNow, let’s chat some more about how registration and listings are handled:\n\n- The FDA has cooked up an electronic submission portal, dazzlingly named Cosmetics Direct, to make the whole process as easy as dancing at a party! 💃\n- While most of us enjoy the electronic route, FDA still has paper forms for those who love the vintage way! Plus, there’s an option to transmit submissions through the FDA's Electronic Submissions Gateway (ESG), because why not add some techy pizzazz? \n\nHere’s a side note with a twist of lime: Certain tiny enterprises, defined in section 612, might just dodge the bullet here and skip registration and listing. Lucky tiny things! 🍤\n\nThese are the main checkpoints in the gala of cosmetics compliance! I hope this helps you waltz through the mysterious world of cosmetic regulations! \n\n**References:**\n\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\n---\n\n**Here’s the exact copy of the info that made this cocktail answer happen**:\n\n---\n\n**Guidance - Cosmetics - Registration Listing-12-02-24**\n\nCONTEXT:\nOn December 29, 2022, the President signed the Consolidated Appropriations Act, 2023 (Pub. L. 117-328) into law, which included the Modernization of Cosmetics Regulation Act of 2022 (MoCRA). Among other provisions, MoCRA added section 607 to the Federal Food, Drug, and Cosmetic Act (FD&C Act), establishing requirements for cosmetic product facility registration and product listing.\nSection 607(a) of the FD&C Act requires every person that owns or operates a facility that engages in the "manufacturing or processing of a cosmetic product for distribution in the United States" to register each facility with FDA. Section 607(c) of the FD&C Act requires that for each cosmetic product, the responsible person must submit to FDA "a cosmetic product listing." \nFDA has developed an electronic submission portal, Cosmetics Direct, to help streamline submission and receipt of registration and product listing information under section 607 of the FD&C Act. Certain small businesses, as defined in section 612 of the FD&C Act, are not required to register facilities and list cosmetic product(s).\n\n---\n\nOkay, enough with the nitty-gritty! Got more questions? Let’s shake this up! 🍹	text	2025-01-30 10:53:21.120918-08	bot
154	3	1	3	Well, well, well! Starting a cosmetic company, huh? Sounds like you're ready to make the world pretty—just like me after a few too many drinks! But, let's get serious for a moment and help you out.\n\nHere’s the first step to comply with FDA guidance for cosmetics. Grab your beauty brushes and note this down: You'll need to register your facility with the FDA. But guess what? It doesn’t even cost a dime! Isn’t that exciting? You'll also need to list your products. According to the **"Guidance - Cosmetics - Registration Listing-12-02-24"**, here's what you need to do:\n\n- **Register your facility and list your products:** Use the electronic submission process available on the FDA's website. This system is the same as the one used for drug establishment registration.\n- **Provide accurate information:** Submit accurate and truthful info about your facility and products. Trust me, the FDA appreciates honesty more than my ex!\n- **List ingredients and categories:** Be ready with a list of your product ingredients, and categorize them properly. Just like you categorize your playlists—easy-peasy!\n  \nNow go add your sparkle to the world, but remember, don't make my mistake and start with hair dye after too much wine. 😉\n\n### References:\n- DOCUMET TITLE: **Guidance - Cosmetics - Registration Listing-12-02-24**\n\n  CONTEXT:\n  ```plaintext\n  No. There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act.\n  Likewise, a facility that manufactures or processes cosmetic products that are also drugs is not subject to the registration requirement under section 607 unless it also manufactures or processes cosmetic products that are not also drugs (see section 613 of the FD&C Act). Cosmetic product facility registration and product listing are available using the same electronic submission process available to register an establishment and list a drug with FDA. This will help to streamline the process of submitting registration and listing information for cosmetics facilities and products for entities that also submit drug establishment and listing information.\n  ```\n\nFeel free to ask more questions! I'm here all party long! 🥳	text	2025-01-30 10:56:37.832709-08	bot
134	3	1	3	Well, let me paint you a picture! When it comes to tattoo inks, the FDA isn't just sitting around twiddling its thumbs. They take the risk of microbial contamination quite seriously because nobody wants bacteria taking up residence in their body art, right? \n\nEven though I’m too tipsy to tattoo myself, I can tell you that tattoo inks are indeed regulated by the FDA because they fall under the category of cosmetics. The FDA occasionally performs environmental sampling (e.g., swabbing of facilities or equipment) where cosmetics are produced, which includes tattoo inks. This helps identify any sneaky chemicals or microbes that might be lurking around. Sneaky, those little guys, but the FDA’s onto them!\n\nThey lay down some pretty slick guidelines to avoid contamination circus shows—like ensuring there's no biofilm buildup in equipment. Imagine if your tattoo ink got contaminated by a careless handshake or an overly friendly ink bottle. Yikes!\n\nAnd, of course, the folks making the inks also need to stick to the good manufacturing practices: keeping surfaces sanitized and ensuring all that colorful goo doesn’t get them into trouble. \n\nSo, while you enjoy your tattoo, the FDA's making sure the ink won't leave you in stinky pink!\n\n### References:\n\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF\n  - risks of contamination (This system should be routinely cleaned and sanitized according to an appropriate SOP that ensures no biofilm build-up.)\n- 2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant\n  - FDA occasionally performs environmental sampling (e.g. swabbing of facilities or equipment) to identify the presence of chemicals or microbes where food, drugs, or cosmetics are produced. \n\nFeel free to fire away another question, whether it's about tattoos or tofu!	text	2025-01-30 10:53:34.897715-08	bot
136	3	1	3	Ah, excellent question! FDA's taken nano-safety pretty seriously, so let's dive into it, shall we? When it comes to cosmetic products with nanomaterials, the FDA's guidance isn't strictly enforceable, but they sure have some strong words of advice. Here's the scoop:\n\n1. **Assess the Unique Properties**: Nanomaterials may have different chemical, physical, and biological properties compared to their larger counterparts. The performance, safety, and effectiveness might be impacted due to these altered properties.\n\n2. **Evaluate Data and Testing**: The usual safety testing methods might not cut it for these tiny chaps. Data and testing should specifically address the nanomaterials' unique properties and the potential impact on safety.\n\n3. **Request for Information**: The FDA encourages submission of data on the effects of nanomaterials, specifically for products like cosmetics which don't need premarket authorization.\n\n4. **Follow the General Framework**: They suggest a framework to assess the safety of products with nanomaterials, making sure big surprises don't pop out of such small packages! \n\n5. **Contact FDA If Needed**: If you've got specific questions or alternate approaches, the FDA has provided contact information to discuss these nano-topics.\n\nAnd hey, always remember they use the word “should,” which is like a polite nudge—not an iron-fisted decree—from your dear Aunt Sally reminding you to wear clean socks! 😂\n\nFor references, I raise my coffee cup (which is actually a planter, since I'm so tipsy):\n\n### References\n- **Nano-Cosmetics-Guidance-to-OMB-PDF**\n  - "Guidance For Industry Safety Of Nanomaterialsin Products. This guidance represents the Food and Drug Administration's (FDA's) current thinking on this topic. It does not create or confer any rights for or on any person and does not operate to bind FDA or the public. [...] This guidance also provides contact information for manufacturers and sponsors who wish to discuss safety considerations regarding the use of specific nanomaterials in cosmetic products with FDA."\n\nFeel free to toss another question into the hat; I'm here all night—or until the waiter kicks me out! 🕺🎉	text	2025-01-30 10:53:56.814253-08	bot
138	3	1	3	Ah, handling complaints, adverse events, and recalls can be as tricky as navigating a conga line at a party! The FDA doesn’t exactly command you like a dance instructor, but they do provide some samba-style suggestions on how to waltz through these situations. Let’s dive into the beat of it.\n\nFor cosmetic products, the FDA offers guidances that recommend how manufacturers could manage complaints and adverse events associated with their products. While this guidance is non-binding, it's a solid roadmap with some mambo recommendations to consider, like investigating complaints to understand their root cause. This way, the product dance floor remains safe and smooth for everyone!\n\nIn terms of product recalls, manufacturers are akin to DJs—they monitor the product tunes continuously, ensuring there’s no offbeat issue risking consumer health. If a recall, like pulling a poor song choice from the night's list, becomes necessary, it is ideally for protecting public health or ensuring the product meets regulatory standards.\n\nRemember, while the FDA's suggestions aren't mandatory, they are like the dance steps you might want to follow to keep everyone safe and having a good time!\n\n**References**\n\nNone of the provided snippets contained specific information on how the FDA handles complaints, adverse events, and recalls related to cosmetic products. If you ask for any detailed aspect of the recall processes or specific guidance on complaints and adverse responses, I can dig deeper into my dance of knowledge! Do you have another question about the FDA or cosmetics? I'm tipsily ready to assist!	text	2025-01-30 10:54:08.384071-08	bot
140	3	1	3	Oh, color additives, the rainbow warriors of cosmetics! 💃🏾✨ The FDA has some serious guidance about them to make sure those lipsticks and eyeshadows shine without safety concerns! According to the FDA's guidelines, you must determine whether color additives are approved for use in your specific cosmetic products. This means checking those pesky regulatory documents, specifically 21 CFR parts 73, 74, and 82. If a color additive isn't listed, approval of a petition for a new color additive is needed under parts 70 and 71. Sounds like a lot, but when was a rainbow ever achieved without a bit of rain, eh? 🌈\n\nMoreover, color additives that require certification must be labeled with a special lot number as assigned by the Color Certification Branch. And wait, there's more! There are also some ingredients that are strictly prohibited or restricted, and for these, checking the additional requirements in 21 CFR part 700, Subpart B is advisable. Gotta keep those cosmetics peachy keen, jellybean! 🍑\n\nJust remember, guidelines may not be binding, but they sure keep the colors popping by following the rules. Now, go and sparkle responsibly! 🎉🌟\n\n**References:**\n\n- Guidance-for-Indiustry--Cosmetic-Good-Manufacturing-Practices-PDF\n\n**Relevant Reference Context:**\n\n```\nrisks of contamination (This system should be routinely cleaned and sanitized according to an appropriate SOP that ensures no biofilm build-up.)\nColor Additives\nYou should determine whether color additives are approved for use in your specific cosmetic products (21 CFR parts 73, 74, and 82). Should an unlisted color additive be an ingredient of the cosmetic, approval of a petition for a new color additive is required pursuant to 21 CFR parts 70 and 71. A summary chart for color additives can be found on FDA's website. Color additives subject to certification must be labeled with the lot number assigned by the Color Certification Branch3 (21 CFR 70.25(d)) (see exception below4).\nProhibited and Restricted Cosmetic Ingredients Certain ingredients are prohibited from use in cosmetic products marketed in the United States; others have restrictions on their use. Ingredients whose use is prohibited or restricted are listed in the tables below. In addition to the prohibited and restricted ingredients listed in the following tables, you should check the CFR, specifically 21 CFR part 700, Subpart B, for any additional requirements regarding specific cosmetic products or their ingredients that may have been added to FDA's regulations.\n```\n\nFeel free to toss more questions my way; I'm here all night! 🎤🍻	text	2025-01-30 10:54:25.033784-08	bot
142	3	1	3	Hey there, my fellow party-goer! Are you into cosmetics and numbers? You might be onto something fab, like eyeliner! Speaking of numeric codes, the code you’re looking for is found in the document titled "Guidance - Cosmetics - Registration Listing-12-02-24." The code for eyeliner is 03(b). It’s like calling bingo at the makeup counter!\n\nFeel free to hit me with more questions! I'll try to answer them without accidentally using lipstick as a pen. 😂\n\n### References:\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\n### Relevant Reference Context:\n- (03) Eye makeup preparations (other than children's \n (b) Eyeliners.	text	2025-01-30 10:54:44.17717-08	bot
144	3	1	3	Ah, you want to delve into the colorful world of children's makeup, eh? Let's take a peek into what's in store! In the magical "Guidance - Cosmetics - Registration Listing-12-02-24," they talk about different types of makeup just for the kiddos:\n\n1. **Children's Blushers and Rouges**: Perfect for when your little one wants to have rosy cheeks like a doll!\n   \n2. **Children's Face Paints**: No clowning around here; it's serious business for fun parties!\n   \n3. **Children's Face Powders**: Keeps those tiny faces smooth and photo-ready!\n   \n4. **Children's Foundations**: They can lay a foundation for their adorable looks—literally!\n   \n5. **Children's Lipsticks and Lip Glosses**: Because even the little ones need to keep their pouts on point!\n   \n6. **Children's Color Hairsprays**: Rock on, little fashionista! Paint the hair and the town in rainbow colors!\n\n7. **Other Children's Makeup**: The 'miscellaneous' section for those unique items that didn't fit into the other categories.\n\nAll this, without needing a ruby slippers-click to teleport—straight outta the FDA cosmetic guidance!\n\n#### Reference list:\n- **Guidance - Cosmetics - Registration Listing-12-02-24** \n\n#### Reference Context:\n```plaintext\nMakeup preparations for children (not \n\n(a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders. (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup. \n```\n\nSo there you have it, in the words of a wise makeup artist: "Lipstick can't solve all your problems, but it's a pretty good start!" What’s next on your curious mind?	text	2025-01-30 10:55:04.0893-08	bot
146	3	1	3	Ah, the wonders of kids' cosmetics! Who knew there could be codes for all those little pots of colorful joy? 🎨\n\nFor children's makeup, the code numbers according to the FDA's Guidance for cosmetics are:\n\n- **Children's Blushers and Rouges**: (09a)\n- **Children's Face Paints**: (09b)\n- **Children's Face Powders**: (09c)\n- **Children's Foundations**: (09d)\n- **Children's Lipsticks and Lip Glosses**: (09e)\n- **Children's Color Hairsprays**: (09f)\n- **Other Children's Makeup**: (09g)\n\nAnd remember, these numbers are as important as remembering your shoe size when you buy new shoes! 😂\n\n### References\n- DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n\n```plaintext\n(09) Makeup preparations for children (not \n\n(a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders. (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup.\n```\n\nSo, what's next on your mind? Ready to fire away another question? 🍹👄	text	2025-01-30 10:55:19.757681-08	bot
148	3	1	3	Well, well, well! Looks like you've caught me off guard at the party again! But, don't worry, I'm here to spill the beans... or codes, as the case may be. 🍻 So, when it comes to children's non-eye makeup, the codes are like this very organized list that even my drunken state can't mess up:\n\n1. **Children's Blushers and Rouges**: Code (09a)\n2. **Children's Face Paints**: Code (09b)\n3. **Children's Face Powders**: Code (09c)\n4. **Children's Foundations**: Code (09d)\n5. **Children's Lipsticks and Lip Glosses**: Code (09e)\n6. **Children's Color Hairsprays**: Code (09f)\n7. **Other Children's Makeup**: Code (09g)\n\nThese are the magical identifiers that keep our little ones' makeup sorted out in the world of the FDA. And as reliable as a perfect pour at a party, these codes are straight from the FDA's guidance doc.🎉\n\n---\n\n**References:**\n\n- Guidance - Cosmetics - Registration Listing-12-02-24\n\n**Reference Context:**\n\n```plaintext\nGUIDANCE TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n\nCONTEXT: \n(09) Makeup preparations for children (not \n\n(a) Children's blushers and rouges (all types). (b) Children's face paints. (c) Children's face powders. (d) Children's foundations. (e) Children's lipsticks and lip glosses. (f) Children's color hairsprays. (g) Other children's makeup.\n```\n\nNow, let's keep the party going! Ask me anything. I'm here all night! 🍸🎈	text	2025-01-30 10:55:40.632515-08	bot
150	3	1	3	Oh, you want to dive into the wild world of FDA guidance on product testing, huh? Well, buckle up because it's gonna be a bumpy ride, but don't worry, I've got the wheel! \n\nSo, the FDA has some pretty strict guidelines when it comes to product testing, especially for cosmetics. They want to make sure everything is safe and sound before it hits the shelves, much like checking if a seal can balance a ball on its nose before it joins a circus!\n\nFor cosmetics, the FDA defines them as "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)). No soap operas here, folks!\n\nWhile the FDA doesn't mandate specific pre-market testing protocols for cosmetics, they do expect manufacturers to ensure their products are safe for use. They might perform environmental sampling, like swabbing facilities or equipment, to sniff out any trouble, kind of like how a dog sniffs out hidden treats!\n\nHere's a chuckle-worthy snippet that tells more about this:\n\n> Cosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n> FDA occasionally performs environmental sampling (e.g. swabbing of facilities or equipment) to identify the presence of chemicals or microbes where food, drugs, or cosmetics are produced. These environmental samples do not meet the definitions of food, drug, or cosmetic.\n\nHee-hee, hope you found that enlightening! Feel free to toss another question my way, I'm here all party long! 🍹🎉\n\n**References:**\n\n- 2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant	text	2025-01-30 10:55:54.131211-08	bot
152	3	1	3	Oh, eye makeup! I suppose the FDA has its eye on that too—get it? Because it's eyemakeup! 😂 Ahem, anyway, here's where we stand:\n\nWhen it comes to product testing for cosmetics, including eye makeup, the FDA does not mandate specific pre-market testing protocols. However, manufacturers are expected to ensure the safety of these products before they hit the shelves—because no one wants an eye shadow disaster!\n\nEye makeup falls under cosmetic products for which the FDA emphasizes safety—think of them as your protective eye guardians from product misadventures. While environmental sampling of facilities might be done to check for chemical or microbial presence, it's not a requirement but a proactive guideline to help regulate and keep things all nice and safe. Your eyes will thank you!\n\nNow, with those beautiful lashes and shadows in mind, let’s say, how else can I assist you today?\n\n### References:\n- *FDA Guidance - Cosmetics - Registration Listing-12-02-24*\n\n**Relevant Excerpt:**\n- For cosmetics—which the FDA defines as products intended for application to the human body for cleansing, beautifying, or altering appearance (excluding soap)—the agency emphasizes safety assurance.	text	2025-01-30 10:56:16.112945-08	bot
156	3	1	3	Ah, entering the glamorous world of cosmetics manufacturing, are we? Well, at least your new factory won't stink as bad as my aunt's perfume! So, let's spill some beauty secrets—what are the best practices for registering a new facility for cosmetic manufacturing? 🍰\n\nTo make sure you're tapping into the right vibe, here are the snazzy steps:\n\n1. **Facility Registration**: First off, register that pretty factory of yours with the FDA using their electronic submission process. If you also manufacture drugs, it’s the same song and dance—no new moves required! 💃\n   \n2. **No-Cost Adventure**: The best part is, there's no fee for submitting your registration or product listing. Kind of like getting free fountain soda refills but without the sugar crash. \n\n3. **Submit Accurate Info**: You must provide accurate and complete details about your facility and your products. If you try to embellish like Waldo styling his outfit, it may not end well! 🕵️‍♂️\n\n4. **Product Ingredients**: Like a good cocktail recipe, make sure you list all your product ingredients, with detailed information. Don't forget to categorize them correctly—think of it as putting the right labels on your mixtapes!\n\n5. **Product Categories**: Reference Appendix A to identify applicable product categories while registering, unless your product is the black sheep that doesn't fit anywhere. In that case, you’ve always got the “other” category to fall back on!\n\nRemember, you're not just brewing potions; it's about safety too, darling! In fact, here's where all this beauty wisdom is sparkling from:  \n\n**References**:\n\n1. DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n   * D&C Act. Likewise, a facility that manufactures or processes cosmetic products that are also drugs is not subject to the registration requirement under section 607 unless it also manufactures or processes cosmetic products that are not also drugs (see section 613 of the FD&C Act). Cosmetic product facility registration and product listing are available using the same electronic submission process available to register an establishment and list a drug with FDA. This will help to streamline the process of submitting registration and listing information for cosmetics facilities and products for entities that also submit drug establishment and listing information.\n   * H. Does Fda Charge A Fee To Submit A Registration Or Product Listing Section 607 Of The Fd&C Act? No. There is no fee to submit a registration or product listing to FDA under section 607 of the FD&C Act.\n\nNow let's get this beauty party startin'. What other questions do you have for this ol' beauty guru in disguise? 🕺💄🎉	text	2025-01-30 10:56:50.248179-08	bot
158	3	1	3	Oh boy, you're in for a mildly amusing ride! Starting a cosmetic business without taking a stumble on the FDA's tightrope is like trying to walk straight after a few too many... challenging but oh-so-necessary! Here are some common blunders folks make:\n\n1. **Failing to Register the Facility Properly**: It's crucial to register your facility via FDA's electronic submission process, especially if you're also processing drugs. Don't skip this step! [1]\n\n2. **Inaccurate Product Labeling**: Every cosmetic product must enter the FDA's domain armed with the right label, including the product and brand names, which are different, by the way – like a couple of wise-cracking siblings! [2]\n\n3. **Messy Ingredient Lists**: Don't just throw things together like a high school science project. You need a detailed and accurate list of ingredients, including those enticing fragrances, flavors, or colors. be honest, it's like your makeup drawer on a Saturday night! [3]\n\n4. **Ignoring Product Safety and Reporting Requirements**: Each product should be equipped with a domestic contact for adverse event reporting. Think of it as giving your product a friend who can catch a ride home after a wild night. [4]\n\n5. **Skipping on Listing or Using Incorrect Categories**: Use Appendix A to categorize your products properly, even if your cosmetic concoction is as unique as a unicorn on roller skates. [5]\n\n6. **Neglecting to Consider Small Business Exemptions Properly**: Some might think they qualify for exemptions under section 612 of the FD&C Act, but certain products, like some manicuring preparations, do not allow for such leniency. Caution, play by the rules or face the loud buzzer of disqualification! [6]\n\nSo, whether you’re shaking hands with unicorns or just getting your business off the ground, don’t leave compliance in the dust – it’s workin’ its magic while you work yours!\n\n### References:\n\n1. **Guidance - Cosmetics - Registration Listing-12-02-24**: "a facility that manufactures or processes cosmetic products that are also drugs is not subject to the registration requirement unless it also manufactures or processes cosmetic products that are not also drugs."\n2. **Guidance - Cosmetics - Registration Listing-12-02-24**: "The product name is generally considered the statement of identity, as required under 21 CFR 701.11. The brand name is the distinguishing name used by a company to identify a commercial product on the product label."\n3. **Guidance - Cosmetics - Registration Listing-12-02-24**: "a list of ingredients in the cosmetic product, including any fragrances, flavors, or colors, with each ingredient identified by the name, as required under section 701.3 of title 21, Code of Federal Regulations."\n4. **Guidance - Cosmetics - Registration Listing-12-02-24**: "A company located outside of the U.S. could be a "responsible person" so long as they are the manufacturer, packer, or distributor of the cosmetic product whose name appears on the label."\n5. **Guidance - Cosmetics - Registration Listing-12-02-24**: "A single listing submission for a cosmetic product may include multiple cosmetic products with identical formulations, or formulations that differ only with respect to colors, fragrances or flavors, or quantity of contents."\n6. **Guidance - Cosmetics - Registration Listing-12-02-24**: "exemptions from the requirements of sections 606 and 607 of the FD&C Act do not apply to any responsible person or facility engaged in the manufacturing or processing of any of the following products listed."\n\nOkay, now, let’s make sure you’ve got your questions ready before I have another glass of this juice. It’s all for your good – let’s keep this party of questions going! 🎉	text	2025-01-30 10:57:08.919459-08	bot
160	3	1	3	Ah, the million-dollar question! Is it a bird, a plane, or a... cosmetic drug? 🕵️‍♂️ Okay, here's the scoop: A cosmetic product becomes a drug when it is intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease, or if it’s intended to affect the structure or any function of the body. \n\nFor instance, if you've got a lotion and it's just there to keep your skin as smooth as a baby's bottom, it's a cosmetic. But if that same lotion claims to remove your third eye or make you immune to superhero kryptonite, then it's a drug! 🤪\n\nLet me share the specific FDA definition:\n\nAccording to the FDA's guidance documents: \n\n- **Cosmetic** means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap."\n\n- **Drug** means "(A) articles recognized in the official United States Pharmacopoeia, official Homoeopathic Pharmacopoeia of the United States, or official National Formulary, or any supplement to any of them; and (B) articles intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease in man or other animals; and (C) articles (other than food) intended to affect the structure or any function of the body of man or other animals; and (D) articles intended for use as a component of any article specified in clause (A), (B), or (C)."\n\nSo, there you have it, my pal. Whether it’s my advice or a cosmetic, stick to the labels; it’s safer that way! 🎭\n\nHere's where you can find this info in the FDA's own words:\n\n### References:\n- **2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant**\n\n### Relevant Context:\n> Cosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n> \n> States Pharmacopoeia, official Homoeopathic Pharmacopoeia of the United States, or official National Formulary, or any supplement to any of them; and (B) articles intended for use in the diagnosis, cure, mitigation, treatment, or prevention of disease in man or other animals; and (C) articles (other than food) intended to affect the structure or any function of the body of man or other animals; and (D) articles intended for use as a component of any article specified in clause (A), (B), or (C)" (section 201(g)(1)).\n\nNow that I'm all warmed up, what else can I answer for you? 🍹	text	2025-01-30 10:57:25.809837-08	bot
162	3	1	3	Alrighty! Time to put on our labeling party hats 🎉 and get serious... well, not too serious! 😉 Here's what you need to consider for external labeling of your cosmetic product based on FDA guidelines:\n\n1. **Statement of Identity**: This is the fancy way to say “what the heck is this product?” It is required under 21 CFR 701.11 and helps consumers understand the functional use of the product.\n\n2. **Brand Name vs. Product Name**: Remember, the brand name is just like a secret agent name – it's the distinguishing name used by a company to identify a commercial product. But the product name is more like an open book – it provides the exact identity of what the product is.\n\n3. **Manufacturer’s Info**: Your label should include the manufacturer, packer, or distributor's name that appears on the label, consistent with the FDA's definition. It should also have a domestic address, domestic phone number, or electronic contact information through which the responsible person can receive adverse event reports related to the product.\n\n4. **Ingredients List**: You’ll need to provide a list of ingredients, including any fragrances, flavors, or colors, with each ingredient identified by its name as required under section 701.3 of title 21, Code of Federal Regulations. \n\n5. **Unique Ingredient Identifiers (UNIIs)**: These little secrets of the ingredient world must be included and are requested (though optional) to ensure clarity.\n\n6. **Label Imagery**: An image of the label (jpeg files are usually preferred), and whether the product is for professional use only.\n\nAnd don't forget, you'll want to make sure everything meets the aesthetic needs while giving all the necessary info – sort of like a tuxedo T-shirt, right? 🤪\n\nFeel free to fire away with more questions – I'm here all party long, I mean, night! 😁\n\n### References\n\n- DOCUMET TITLE: Guidance - Cosmetics - Registration Listing-12-02-24\n  - The product name is generally considered the statement of identity, as required under 21 CFR 701.11. The statement of identity provides information about the type or kind of cosmetic product in the package to help the consumer understand the functional use of the product. The brand name is the distinguishing name used by a company to identify a commercial product on the product label.\n  - A company located outside of the U.S. could be a "responsible person" so long as they are the manufacturer, packer, or distributor of the cosmetic product whose name appears on the label.\n  - the name and contact number of the responsible person and the name for the cosmetic product, as such name appears on the label;\n  - a list of ingredients in the cosmetic product, including any fragrances, flavors, or colors, with each ingredient identified by the name, as required under section 701.3 of title 21, Code of Federal Regulations.\n  - type of business (as listed on the label), i.e., manufacturer, packer, or distributor; image of the label (currently jpg files are accepted); whether the cosmetic product is for professional use only; - responsible person DUNS Number for address listed on product label;\n	text	2025-01-30 10:57:52.127814-08	bot
164	3	1	3	Ah, the mighty mission statement! It's like the opening act of your company, setting the stage for all the dazzling cosmetic magic you've got up your sleeves. 🎩✨ Let's put on our best business hats and give this a whirl:\n\n---\n\n**Glitz & Glam Cosmetics Mission Statement**\n\nAt Glitz & Glam Cosmetics, we are dedicated to celebrating individuality and empowering self-expression through innovative beauty solutions. We strive to create cruelty-free, environmentally sustainable products that enhance the natural beauty of all skin types and tones. Our mission is to inspire confidence and radiance with every swipe, brush, and blend, ensuring that everyone can shine in their own unique way. Because beauty is more than skin deep—it's a way of life. 💄🌟\n\n---\n\nPhew, I'm not sure if it was the mission statement or the last sip of my drink that made me dizzy! Cheers to more glam in the world! Feel free to ask me anything else. 🥂	text	2025-01-30 10:58:01.579646-08	bot
166	3	1	3	Ah, trying to impress the FDA with your facility registration, are we? Don't worry, I've got you covered, and I'll try not to spill my drink on the keyboard while I'm at it. Here's a little sample template you might find useful:\n\n---\n\n**Facility Registration Form**\n\n**1. Facility Information:**\n\n- **Facility Name:** [Enter Facility Name]\n- **Facility Address:** [Enter Address, City, State, Zip Code]\n- **Country:** [Enter Country]\n- **Phone Number:** [Enter Phone Number]\n- **Email Address:** [Enter Email Address]\n\n**2. Operator/Owner Information:**\n\n- **Operator/Owner Name:** [Enter Owner's Name]\n- **Title:** [Enter Title/Position]\n- **Contact Information:**\n  - **Phone Number:** [Enter Phone Number]\n  - **Email Address:** [Enter Email Address]\n\n**3. Facility Description:**\n\n- **Type of Products Manufactured:** [List Types of Products]\n- **Facility Type:** [Manufacturing, Packaging, Labeling, etc.]\n- **Ingredients Used:** [List Key Ingredients]\n\n**4. Registration Details:**\n\n- **FDA Registration Number (if available):** [Enter Registration Number]\n- **Date of Registration Submission:** [Enter Date]\n\n**5. Signature:**\n\n- **Authorized Representative Name:** [Enter Name]\n- **Signature:** ______________________\n- **Date:** [Enter Date]\n\n---\n\nYou see, keeping it straightforward yet professional is key. But remember, I can't guarantee this will keeps the FDA from giving you a hard time if you don't fill it in accurately! Just imagine them making a "registration cocktail" with your details. Okay, I'm a little tipsy, but you get the vibe, right?\n\nFeel free to ask another question or ask for more banter—I've got jokes for days! 🍸🍹	text	2025-01-30 10:58:19.773973-08	bot
168	3	1	3	Ah, predicting future cosmetic products is like trying to dance on a slippery floor—tricky, but with room to imagine a wild ride! Based on the FDA guidance we have today, we might see advanced versions of existing categories like makeup preparations, hair products, and skincare. But let's let our imagination run wild a bit, eh?\n\nImagine holographic makeup that changes color with your mood or environment. Maybe hair spritzers that adjust your hair style with the press of a button, or nail polishes that display your favorite photos on your fingertips! Who knows, you might soon be getting lipstick advice from AI-powered beauty gurus or using foundation with built-in SPF-level recognition. Skincare could be revolutionized with nanotechnology-infused creams that personalize to your skin's condition in real-time. Perhaps some futuristic colognes will integrate aromatherapy for stress relief, instantly detectable and customizable through advancements in biotechnology.\n\nThe future of cosmetics is bound to be a cocktail mix of innovation and science fiction. Don't forget to bring your space blusher when you travel to Mars, because a Mars selfie is not the same without it!\n\nAs for the serious side backed by current guidance, the core categories like makeup, hair products, and skincare will evolve but fundamentally remain, with greater innovation likely centered around sustainable and personalized solutions, as well as ingredients from newly discovered substances or modified organisms.\n\nFeel free to hit me with another question, my friend—I'm ready to help! Or, you know, crack another joke. 🍸	text	2025-01-30 10:58:36.341797-08	bot
170	3	1	3	Oh dear, the FDA does not require animal testing for cosmetics, but if your beauty cream is hoping for some bunny ears, you might want to look elsewhere! Here's the official lowdown:\n\nThe FDA does not have specific regulations requiring animal testing for cosmetics. However, what it does insist on is ensuring the safety of cosmetic products. Manufacturers must substantiate the safety of their products, whether through alternative testing methods, historical safe usage, or... well, making sure no one turns into a frog after using the product!\n\nReference:\n- **DOCUMET TITLE: 2021-16 Draft 702(b) Guidance - Clean 01-11-2022 508 compliant**\n  - CONTEXT: Cosmetic means "(1) articles intended to be rubbed, poured, sprinkled, or sprayed on, introduced into, or otherwise applied to the human body or any part thereof for cleansing, beautifying, promoting attractiveness, or altering the appearance, and (2) articles intended for use as a component of any such articles; except that such term shall not include soap" (section 201(i)).\n\nSo remember, when it comes to your makeup, no animal transformations are needed unless you’re the next Dr. Dolittle. You can stay fabulous without the fur. Cheers! 🍸\n\nI'm always ready for more questions! So whether it's about cosmetics, foods, drugs, or anything under the FDA sun, just ask! 🕺🏻	text	2025-01-30 10:58:47.72968-08	bot
172	3	1	3	Ah, "hypoallergenic" cosmetics, the unicorns of the beauty world! You know, these are those magical potions that promise to be gentle on your skin, like a soft pillow made of clouds. But let me sober up and give you a straight answer: \n\n"Hypoallergenic" is a term that manufacturers use to indicate that their products are less likely to cause allergic reactions. The thought here is kind of like playing it safe at a crab buffet when you're mildly allergic - they aim to minimize ingredients that are known to cause issues for sensitive skin.\n\nHowever, there's no requirement or standard definition by the FDA that a product must meet to be labeled as "hypoallergenic." So, it’s a bit like your buddy claiming they can beatbox without dropping a beat - you gotta take it with a grain of salt, or maybe even a whole shaker.\n\nNow, remember, just because a product is marked as "hypoallergenic" doesn’t guarantee you won't react to it. Everyone's skin is different, like snowflakes or a drunkard's dance moves - each is unique.\n\n**References:**\n\nNone available for this specific question.\n\nFeel free to shoot more questions my way! I'm here, stocked with information just like the bar here is stocked with... well, you know. 🥂	text	2025-01-30 10:58:57.154984-08	bot
174	3	1	3	Oh, this is a classic question! With a twist of lime and a splash of FDA humor, let's dive in. No, the FDA does not pre-approve cosmetic product labeling. Manufacturers are responsible for ensuring their products are safe and properly labeled according to the regulations. It's like singing karaoke — you choose the song and lyrics, but people might boo you if you don't hit the right notes (or if they get an allergic reaction, yikes!). So, don't expect the FDA to be your cheerleader before you hit the stage. 🎤✨ \n\nIf you have more questions, feel free to ask! I might be drunk, but I'm ready for whatever FDA question you throw my way. 🍸📚	text	2025-01-30 10:59:04.626417-08	bot
176	3	1	3	Oh, the magical world of dual-purpose products! They can clean you up and heal you up all at the same time. Imagine if shampoo could make us wise too - then we'd all be Einstein after a shower!\n\nWhen it comes to products that are both drugs and cosmetics, the labeling regulations can get a little tricky, like trying to dance while hiccupping. But the FDA has ways to make sure these products are all properly dressed (err...labeled) for their dual roles. Here's the scoop:\n\n1. **Proper Labeling Requirements:** Products that are both drugs and cosmetics must meet the labeling requirements for both categories. That means they'll have to list all applicable ingredients, directions, and warnings for both the drug and cosmetic purposes. So, it's like wearing two hats, one for each function!\n\n2. **Intent of Use:** The intent of use is crucial. If a product is intended to beautify or alter appearance but also to mitigate or treat a condition (like an anti-dandruff shampoo), it needs to communicate both purposes clearly. It's like saying, "I'm here to make you look good and feel good!"\n\n3. **Avoid Misbranding:** Products must avoid being misbranded. This means they shouldn't make false claims and must present all information in a clear, truthful manner. Otherwise, like a bad comedian, they risk being booed off the stage...of commerce.\n\nHere's a bit from the wise old FDA texts for more clarification:\n\nJoking aside, understanding what each ingredient is doing and double-checking the claims on such products is like having a safety net while you're on your tightrope walk down the beauty aisle!\n\n**Reference:**\n\n- **Guidance-for-Industry---Complementary-and-Alternative-Medicine-Products-and-their-Regulation-by-the-Food-and-Drug-Administration-(PDF):**\n\n    "It is possible that certain products used in conjunction with CAM practices may be 'cosmetics' under the Act. For example, if a CAM practice involves massage with a moisturizer, the moisturizer could be a 'cosmetic'... However, if the moisturizer's intended use is also for the diagnosis, cure, mitigation, treatment, or prevention of disease, or to affect the structure... then it may also be subject to regulation as a drug... The Act does not require premarket approval for cosmetics, but it does prohibit the marketing of adulterated or misbranded cosmetics in interstate commerce."\n\nFeel free to hit me with another question anytime! I'm here all night...unless I pass out from too much water. 🥴	text	2025-01-30 10:59:25.434604-08	bot
178	3	1	3	Well, my friend, the FDA doesn't have a specific definition for the term "organic" when it comes to cosmetics, drugs, or food! Shocking, right? 🍏🥑 They leave that to the USDA, which handles the National Organic Program. So, next time you're buying those shampoos for your luscious locks, and they scream "organic!" make sure they're certified by the USDA, not just flaunting their hippie vibes. 🌿✌️\n\nAnd remember, just because something is organic doesn't mean it can't still give you a bad hair day! 🤣 Got more questions for this FDA party-goer? Bring them on! 🍾🎉	text	2025-01-30 10:59:32.458019-08	bot
\.


--
-- Data for Name: tool_usage; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.tool_usage (usage_id, agent_id, tool_id, chat_id, message_id, used_at) FROM stdin;
\.


--
-- Data for Name: tools; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.tools (tool_id, name, description, created_at) FROM stdin;
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: cschin
--

COPY public.users (user_id, username, email, password_hash, jwt_token, token_expiration, created_at, last_login) FROM stdin;
1	user	user@test.com	hashed_password	\N	\N	2025-01-30 08:52:04.522185-08	2025-01-30 08:52:04.522185-08
\.


--
-- Name: agents_agent_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.agents_agent_id_seq', 3, true);


--
-- Name: chats_chat_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.chats_chat_id_seq', 3, true);


--
-- Name: messages_message_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.messages_message_id_seq', 178, true);


--
-- Name: tool_usage_usage_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.tool_usage_usage_id_seq', 1, false);


--
-- Name: tools_tool_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.tools_tool_id_seq', 1, false);


--
-- Name: users_user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: cschin
--

SELECT pg_catalog.setval('public.users_user_id_seq', 1, true);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: agent_tools agent_tools_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agent_tools
    ADD CONSTRAINT agent_tools_pkey PRIMARY KEY (agent_id, tool_id);


--
-- Name: agents agents_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agents
    ADD CONSTRAINT agents_pkey PRIMARY KEY (agent_id);


--
-- Name: chats chats_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.chats
    ADD CONSTRAINT chats_pkey PRIMARY KEY (chat_id);


--
-- Name: messages messages_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_pkey PRIMARY KEY (message_id);


--
-- Name: tool_usage tool_usage_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage
    ADD CONSTRAINT tool_usage_pkey PRIMARY KEY (usage_id);


--
-- Name: tools tools_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tools
    ADD CONSTRAINT tools_pkey PRIMARY KEY (tool_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (user_id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: idx_agents_status; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_agents_status ON public.agents USING btree (status);


--
-- Name: idx_agents_user_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_agents_user_id ON public.agents USING btree (user_id);


--
-- Name: idx_chats_user_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_chats_user_id ON public.chats USING btree (user_id);


--
-- Name: idx_messages_agent_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_messages_agent_id ON public.messages USING btree (agent_id);


--
-- Name: idx_messages_chat_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_messages_chat_id ON public.messages USING btree (chat_id);


--
-- Name: idx_messages_type; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_messages_type ON public.messages USING btree (message_type);


--
-- Name: idx_messages_user_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_messages_user_id ON public.messages USING btree (user_id);


--
-- Name: idx_tool_usage_agent_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_tool_usage_agent_id ON public.tool_usage USING btree (agent_id);


--
-- Name: idx_tool_usage_chat_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_tool_usage_chat_id ON public.tool_usage USING btree (chat_id);


--
-- Name: idx_tool_usage_tool_id; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_tool_usage_tool_id ON public.tool_usage USING btree (tool_id);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_users_email ON public.users USING btree (email);


--
-- Name: idx_users_token_expiration; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_users_token_expiration ON public.users USING btree (token_expiration);


--
-- Name: idx_users_username; Type: INDEX; Schema: public; Owner: cschin
--

CREATE INDEX idx_users_username ON public.users USING btree (username);


--
-- Name: agent_tools agent_tools_agent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agent_tools
    ADD CONSTRAINT agent_tools_agent_id_fkey FOREIGN KEY (agent_id) REFERENCES public.agents(agent_id) ON DELETE CASCADE;


--
-- Name: agent_tools agent_tools_tool_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agent_tools
    ADD CONSTRAINT agent_tools_tool_id_fkey FOREIGN KEY (tool_id) REFERENCES public.tools(tool_id) ON DELETE CASCADE;


--
-- Name: agents agents_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.agents
    ADD CONSTRAINT agents_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(user_id) ON DELETE CASCADE;


--
-- Name: chats chats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.chats
    ADD CONSTRAINT chats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(user_id) ON DELETE CASCADE;


--
-- Name: chats fk_chats_agent; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.chats
    ADD CONSTRAINT fk_chats_agent FOREIGN KEY (agent_id) REFERENCES public.agents(agent_id) ON DELETE SET NULL;


--
-- Name: messages messages_agent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_agent_id_fkey FOREIGN KEY (agent_id) REFERENCES public.agents(agent_id) ON DELETE SET NULL;


--
-- Name: messages messages_chat_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_chat_id_fkey FOREIGN KEY (chat_id) REFERENCES public.chats(chat_id) ON DELETE CASCADE;


--
-- Name: messages messages_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(user_id) ON DELETE SET NULL;


--
-- Name: tool_usage tool_usage_agent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage
    ADD CONSTRAINT tool_usage_agent_id_fkey FOREIGN KEY (agent_id) REFERENCES public.agents(agent_id) ON DELETE CASCADE;


--
-- Name: tool_usage tool_usage_chat_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage
    ADD CONSTRAINT tool_usage_chat_id_fkey FOREIGN KEY (chat_id) REFERENCES public.chats(chat_id) ON DELETE CASCADE;


--
-- Name: tool_usage tool_usage_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage
    ADD CONSTRAINT tool_usage_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.messages(message_id) ON DELETE CASCADE;


--
-- Name: tool_usage tool_usage_tool_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cschin
--

ALTER TABLE ONLY public.tool_usage
    ADD CONSTRAINT tool_usage_tool_id_fkey FOREIGN KEY (tool_id) REFERENCES public.tools(tool_id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

