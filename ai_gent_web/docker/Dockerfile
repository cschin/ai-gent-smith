FROM ubuntu:latest AS build
RUN apt-get update && apt-get install -y locales curl && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
RUN locale-gen "en_US.UTF-8"
ENV LANG=en_US.utf8
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y build-essential git zlib1g-dev curl openssl pkg-config libssl-dev postgresql
RUN rm -rf /var/lib/apt/lists/*
RUN mkdir -p /opt/bin && mkdir -p /opt/data && mkdir -p /opt/database_setup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN . $HOME/.cargo/env && cargo install sqlx-cli 
RUN cd /opt && git clone -b main https://github.com/cschin/ai-gent-smith.git 
COPY setup_db.sh /opt/bin/setup_db.sh
ENV DATABASE_URL=postgres://db_user@localhost/ai_gent 
ENV PGPASSWORD=dslakdasd
RUN /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y && apt-get install postgresql-16-pgvector
RUN bash /opt/bin/setup_db.sh && cd /opt/ai-gent-smith/ai_gent_web && . $HOME/.cargo/env && cargo build --release 

FROM ubuntu:latest
RUN apt-get update && apt-get install -y locales curl && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
RUN locale-gen "en_US.UTF-8"
ENV LANG=en_US.utf8
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y ca-certificates postgresql
RUN /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y && apt-get install postgresql-16-pgvector
RUN rm -rf /var/lib/apt/lists/*
RUN mkdir -p /opt/ && mkdir -p /opt/bin && mkdir -p /opt/database && mkdir -p /opt/data
COPY bootstraping.sh /opt/database/
COPY bootstraping.sql /opt/database/
ENV DATABASE_URL=postgres://db_user@localhost/ai_gent 
RUN useradd -ms /bin/bash db_user
COPY load_db.sh /opt/bin/load_db.sh
RUN bash /opt/bin/load_db.sh
ENV PGUSER=db_user
ENV PGPASSWORD=dslakdasd
COPY run.sh /opt/bin/run.sh
COPY --from=build /opt/ai-gent-smith/target/release/ai_gent_web /opt/bin/ai_gent_web
CMD ["bash", "/opt/bin/run.sh"]
