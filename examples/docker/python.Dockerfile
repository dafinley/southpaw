# Example Python app image with southpaw as an entrypoint gate.
# Replace the Southpaw binary source with your preferred install path (prebuilt release, internal artifact, etc.).

FROM python:3.12-slim AS runtime

WORKDIR /app

# Install your app dependencies / copy app code as usual
# COPY requirements.txt .
# RUN pip install --no-cache-dir -r requirements.txt
# COPY . .

# Add a prebuilt southpaw CLI binary into the image.
# Example:
#   COPY southpaw /usr/local/bin/southpaw
COPY southpaw /usr/local/bin/southpaw
RUN chmod +x /usr/local/bin/southpaw

COPY examples/policies/python-runtime.yaml /etc/southpaw/southpaw.yaml

# Run Southpaw first, then replace PID 1 with the Python process (`--exec` uses exec on Unix).
ENTRYPOINT ["southpaw", "check", "--policy", "/etc/southpaw/southpaw.yaml", "--exec", "python", "-m", "your_app"]
