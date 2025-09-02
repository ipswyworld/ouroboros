--
-- PostgreSQL database dump
--

\restrict E1wEJUo09aNIqfpf4Ehq5mov8LCIyohmOqEyZroG5x1freJ5UN0zQQqpw5AE2oQ

-- Dumped from database version 16.10 (Debian 16.10-1.pgdg13+1)
-- Dumped by pg_dump version 16.10 (Debian 16.10-1.pgdg13+1)

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
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO ouro;

--
-- Name: blocks; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.blocks (
    block_id uuid NOT NULL,
    block_height bigint NOT NULL,
    parent_ids uuid[] NOT NULL,
    merkle_root text NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    tx_count integer NOT NULL,
    block_bytes bytea NOT NULL,
    signer text,
    signature text,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.blocks OWNER TO ouro;

--
-- Name: evidence; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.evidence (
    id uuid NOT NULL,
    validator text NOT NULL,
    round bigint NOT NULL,
    existing_block text,
    conflicting_block text,
    reported_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.evidence OWNER TO ouro;

--
-- Name: mempool; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.mempool (
    tx_id uuid NOT NULL,
    transaction_data bytea NOT NULL,
    received_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.mempool OWNER TO ouro;

--
-- Name: mempool_entries; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.mempool_entries (
    tx_id uuid NOT NULL,
    tx_hash text,
    payload jsonb,
    received_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.mempool_entries OWNER TO ouro;

--
-- Name: sbt; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.sbt (
    sbt_id text NOT NULL,
    issuer text NOT NULL,
    meta jsonb,
    mint_tx uuid NOT NULL,
    minted_at timestamp with time zone NOT NULL,
    revoked boolean DEFAULT false,
    revoked_by text,
    revoked_at timestamp with time zone
);


ALTER TABLE public.sbt OWNER TO ouro;

--
-- Name: schema_migrations; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.schema_migrations (
    filename text NOT NULL,
    applied_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.schema_migrations OWNER TO ouro;

--
-- Name: transactions; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.transactions (
    tx_id uuid NOT NULL,
    tx_hash text,
    sender text,
    recipient text,
    payload jsonb,
    status text DEFAULT 'pending'::text,
    included_in_block uuid,
    created_at timestamp with time zone DEFAULT now(),
    idempotency_key text,
    nonce bigint
);


ALTER TABLE public.transactions OWNER TO ouro;

--
-- Name: tx_index; Type: TABLE; Schema: public; Owner: ouro
--

CREATE TABLE public.tx_index (
    tx_hash text NOT NULL,
    tx_id uuid NOT NULL,
    block_id uuid,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.tx_index OWNER TO ouro;

--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
1	create chain schema	2025-08-26 07:45:06.946357+00	t	\\x14ff9e307540a50e5436a90f412a8b46fe7dbc189355d80b30128780478c1a1aafe3510183828c04275936a6f5e5ee76	364433200
2	add idempotency and nonce	2025-08-26 07:45:07.347129+00	t	\\x13f9c0e40e291f0a223be2c8f9d56bf1eaeba3f09fd359a2df93debc2ac41a617272abdb17020c3e2e7749273c6d5573	35422500
3	create evidence table	2025-08-26 07:45:07.391019+00	t	\\x3c18e607c53b4723e5af3360632add84ed38cbc91b386290ed9b852d86a4d64bc02dc007fb0b3057ae26d23de9390f1d	69987300
4	create mempool table	2025-08-26 07:45:07.471437+00	t	\\x153eeda4f3938e8a631e36c90fad69e2af5fa79636d2fee8a54f4715b9ddfbe52726875d1281849b01d8d69ff9b3f11f	58398800
\.


--
-- Data for Name: blocks; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.blocks (block_id, block_height, parent_ids, merkle_root, "timestamp", tx_count, block_bytes, signer, signature, created_at) FROM stdin;
\.


--
-- Data for Name: evidence; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.evidence (id, validator, round, existing_block, conflicting_block, reported_at) FROM stdin;
\.


--
-- Data for Name: mempool; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.mempool (tx_id, transaction_data, received_at) FROM stdin;
\.


--
-- Data for Name: mempool_entries; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.mempool_entries (tx_id, tx_hash, payload, received_at) FROM stdin;
3140cca9-76f1-49a3-b39b-392d008a2477	abc123	{"foo": "bar"}	2025-08-27 12:15:06.247954+00
\.


--
-- Data for Name: sbt; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.sbt (sbt_id, issuer, meta, mint_tx, minted_at, revoked, revoked_by, revoked_at) FROM stdin;
\.


--
-- Data for Name: schema_migrations; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.schema_migrations (filename, applied_at) FROM stdin;
0001_create_chain_schema.sql	2025-08-28 07:16:42.662441+00
0002_add_idempotency_and_nonce.sql	2025-08-28 07:16:42.720645+00
0003_create_evidence_table.sql	2025-08-28 07:16:42.758038+00
0004_create_mempool_table.sql	2025-08-28 07:16:42.798535+00
0005_create_sbt_table.sql	2025-08-28 07:16:42.834758+00
\.


--
-- Data for Name: transactions; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.transactions (tx_id, tx_hash, sender, recipient, payload, status, included_in_block, created_at, idempotency_key, nonce) FROM stdin;
3140cca9-76f1-49a3-b39b-392d008a2477	abc123	A	B	{"foo": "bar"}	pending	\N	2025-08-27 12:15:06.247954+00	\N	\N
\.


--
-- Data for Name: tx_index; Type: TABLE DATA; Schema: public; Owner: ouro
--

COPY public.tx_index (tx_hash, tx_id, block_id, created_at) FROM stdin;
abc123	3140cca9-76f1-49a3-b39b-392d008a2477	\N	2025-08-27 12:15:06.247954+00
\.


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: blocks blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (block_id);


--
-- Name: evidence evidence_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.evidence
    ADD CONSTRAINT evidence_pkey PRIMARY KEY (id);


--
-- Name: mempool_entries mempool_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.mempool_entries
    ADD CONSTRAINT mempool_entries_pkey PRIMARY KEY (tx_id);


--
-- Name: mempool mempool_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.mempool
    ADD CONSTRAINT mempool_pkey PRIMARY KEY (tx_id);


--
-- Name: sbt sbt_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.sbt
    ADD CONSTRAINT sbt_pkey PRIMARY KEY (sbt_id);


--
-- Name: schema_migrations schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.schema_migrations
    ADD CONSTRAINT schema_migrations_pkey PRIMARY KEY (filename);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (tx_id);


--
-- Name: transactions transactions_tx_hash_key; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_tx_hash_key UNIQUE (tx_hash);


--
-- Name: tx_index tx_index_pkey; Type: CONSTRAINT; Schema: public; Owner: ouro
--

ALTER TABLE ONLY public.tx_index
    ADD CONSTRAINT tx_index_pkey PRIMARY KEY (tx_hash);


--
-- Name: idx_blocks_height; Type: INDEX; Schema: public; Owner: ouro
--

CREATE INDEX idx_blocks_height ON public.blocks USING btree (block_height);


--
-- Name: idx_idempotency; Type: INDEX; Schema: public; Owner: ouro
--

CREATE UNIQUE INDEX idx_idempotency ON public.transactions USING btree (idempotency_key) WHERE (idempotency_key IS NOT NULL);


--
-- Name: idx_tx_sender; Type: INDEX; Schema: public; Owner: ouro
--

CREATE INDEX idx_tx_sender ON public.transactions USING btree (sender);


--
-- Name: idx_tx_status; Type: INDEX; Schema: public; Owner: ouro
--

CREATE INDEX idx_tx_status ON public.transactions USING btree (status);


--
-- PostgreSQL database dump complete
--

\unrestrict E1wEJUo09aNIqfpf4Ehq5mov8LCIyohmOqEyZroG5x1freJ5UN0zQQqpw5AE2oQ

