--
-- Name: plays; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.plays (
    id integer NOT NULL,
    recording_id integer NOT NULL,
    program_id integer NOT NULL,
    started_at timestamp with time zone NOT NULL
);

--
-- Name: plays_id_seq; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.plays_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

--
-- Name: plays_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.plays_id_seq OWNED BY public.plays.id;


--
-- Name: program_tags; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.program_tags (
    id integer NOT NULL,
    program_id integer NOT NULL,
    tag_id integer NOT NULL
);


--
-- Name: program_tags_id_seq; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.program_tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: program_tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.program_tags_id_seq OWNED BY public.program_tags.id;


--
-- Name: programs; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.programs (
    id integer NOT NULL,
    name text NOT NULL,
    starts_at time without time zone NOT NULL,
    ends_at time without time zone NOT NULL
);


--
-- Name: programs_id_seq; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.programs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: programs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.programs_id_seq OWNED BY public.programs.id;


--
-- Name: recording_tags; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.recording_tags (
    id integer NOT NULL,
    recording_id integer NOT NULL,
    tag_id integer NOT NULL
);


--
-- Name: recordings; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.recordings (
    id integer NOT NULL,
    filename text NOT NULL,
    title text NOT NULL,
    artist text NOT NULL,
    link text
);


--
-- Name: recordings_id_seq; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.recordings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: recordings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.recordings_id_seq OWNED BY public.recordings.id;


--
-- Name: tags; Type: TABLE; Schema: public; Owner: charlie
--

CREATE TABLE public.tags (
    id integer NOT NULL,
    name text NOT NULL
);


--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.tags_id_seq OWNED BY public.recording_tags.id;


--
-- Name: tags_id_seq1; Type: SEQUENCE; Schema: public; Owner: charlie
--

CREATE SEQUENCE public.tags_id_seq1
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tags_id_seq1; Type: SEQUENCE OWNED BY; Schema: public; Owner: charlie
--

ALTER SEQUENCE public.tags_id_seq1 OWNED BY public.tags.id;


--
-- Name: plays id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.plays ALTER COLUMN id SET DEFAULT nextval('public.plays_id_seq'::regclass);


--
-- Name: program_tags id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.program_tags ALTER COLUMN id SET DEFAULT nextval('public.program_tags_id_seq'::regclass);


--
-- Name: programs id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.programs ALTER COLUMN id SET DEFAULT nextval('public.programs_id_seq'::regclass);


--
-- Name: recording_tags id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recording_tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);


--
-- Name: recordings id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recordings ALTER COLUMN id SET DEFAULT nextval('public.recordings_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq1'::regclass);


--
-- Name: plays plays_pkey; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.plays
    ADD CONSTRAINT plays_pkey PRIMARY KEY (id);


--
-- Name: program_tags program_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.program_tags
    ADD CONSTRAINT program_tags_pkey PRIMARY KEY (id);


--
-- Name: programs programs_pkey; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.programs
    ADD CONSTRAINT programs_pkey PRIMARY KEY (id);


--
-- Name: recordings recordings_filename_key; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recordings
    ADD CONSTRAINT recordings_filename_key UNIQUE (filename);


--
-- Name: recordings recordings_pkey; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recordings
    ADD CONSTRAINT recordings_pkey PRIMARY KEY (id);


--
-- Name: tags tags_name_key; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_name_key UNIQUE (name);


--
-- Name: recording_tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recording_tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: tags tags_pkey1; Type: CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey1 PRIMARY KEY (id);


--
-- Name: program_tags_program_id_tag_id_idx; Type: INDEX; Schema: public; Owner: charlie
--

CREATE UNIQUE INDEX program_tags_program_id_tag_id_idx ON public.program_tags USING btree (program_id, tag_id);


--
-- Name: program_tags_tag_id_idx; Type: INDEX; Schema: public; Owner: charlie
--

CREATE INDEX program_tags_tag_id_idx ON public.program_tags USING btree (tag_id);


--
-- Name: recording_tags_recoding_id_tag_id_idx; Type: INDEX; Schema: public; Owner: charlie
--

CREATE UNIQUE INDEX recording_tags_recoding_id_tag_id_idx ON public.recording_tags USING btree (recording_id, tag_id);


--
-- Name: recording_tags_tag_id_idx; Type: INDEX; Schema: public; Owner: charlie
--

CREATE INDEX recording_tags_tag_id_idx ON public.recording_tags USING btree (tag_id);


--
-- Name: plays plays_program_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.plays
    ADD CONSTRAINT plays_program_id_fkey FOREIGN KEY (program_id) REFERENCES public.programs(id) ON DELETE CASCADE;


--
-- Name: plays plays_recording_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.plays
    ADD CONSTRAINT plays_recording_id_fkey FOREIGN KEY (recording_id) REFERENCES public.recordings(id) ON DELETE CASCADE;


--
-- Name: program_tags program_tags_program_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.program_tags
    ADD CONSTRAINT program_tags_program_id_fkey FOREIGN KEY (program_id) REFERENCES public.programs(id) ON DELETE CASCADE;


--
-- Name: program_tags program_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.program_tags
    ADD CONSTRAINT program_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: recording_tags recording_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recording_tags
    ADD CONSTRAINT recording_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: recording_tags tags_recoding_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: charlie
--

ALTER TABLE ONLY public.recording_tags
    ADD CONSTRAINT tags_recoding_id_fkey FOREIGN KEY (recording_id) REFERENCES public.recordings(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

