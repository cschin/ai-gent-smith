COPY public.users (user_id, username, email, password_hash, jwt_token, token_expiration, created_at, last_login) FROM stdin;
2   jane_smith  jane@example.com    hashed_password_2   \N  \N  2024-09-09 21:54:58.56971-07    2024-09-09 21:54:58.56971-07
3   bob_johnson bob@example.com hashed_password_3   \N  \N  2024-09-09 21:54:58.56971-07    2024-09-09 21:54:58.56971-07
1   test    john@example.com    hashed_password_1   \N  \N  2024-09-09 21:54:58.56971-07    2024-09-09 21:54:58.56971-07
\.
