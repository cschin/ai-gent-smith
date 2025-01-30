FROM ubuntu:latest
RUN apt-get update && apt-get install -y locales && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
ENV LANG en_US.utf8
RUN apt-get install -y build-essential git zlib1g-dev curl openssl pkg-config libssl-dev
RUN apt-get install -y postgresql
RUN rm -rf /var/lib/apt/lists/*
RUN mkdir -p /opt/bin && mkdir -p /opt/data && mkdir -p /opt/database_setup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN . $HOME/.cargo/env && cargo install sqlx-cli 
RUN touch /trigger3
RUN cd /opt && git clone -b fda_challenge https://github.com/cschin/ai-gent-smith.git 
COPY setup_db.sh /opt/bin/setup_db.sh
ENV DATABASE_URL=postgres://db_user@localhost/ai_gent 
ENV PGPASSWORD=dslakdasd
RUN bash /opt/bin/setup_db.sh && cd /opt/ai-gent-smith/ai_gent_web && . $HOME/.cargo/env && cargo build --release 
ENV PGPASSWORD=dslakdasd
ENV DATABASE_URL=postgres://db_user@localhost/ai_gent
COPY all_embedding.jsonl.gz /opt/data
COPY run.sh /opt/bin/run.sh
CMD /bin/bash /opt/bin/run.sh
