# Build from repo root:
# docker build -f examples/languages/python/Dockerfile -t southpaw-python-demo .

FROM rust:1-bookworm AS southpaw-builder
WORKDIR /repo
COPY . .
RUN cargo build -p southpaw-cli --release

FROM python:3.12-slim
WORKDIR /app
COPY --from=southpaw-builder /repo/target/release/southpaw /usr/local/bin/southpaw
COPY bindings/python /opt/southpaw-python
RUN pip install --no-cache-dir /opt/southpaw-python
COPY examples/languages/python/app.py /app/app.py
COPY examples/languages/python/southpaw.yaml /app/southpaw.yaml
RUN useradd --uid 10001 --create-home appuser
USER appuser
ENTRYPOINT ["python", "/app/app.py"]

