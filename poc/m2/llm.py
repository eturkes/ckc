"""llama-server lifecycle + chat HTTP client. Stdlib only."""
import json
import subprocess
import time
import urllib.request
from pathlib import Path


class LlamaServer:
    def __init__(self, server_bin, model_path, port=8077, ctx=4096, threads=8,
                 n_gpu_layers=99):
        self.server_bin = Path(server_bin)
        self.model_path = Path(model_path)
        self.port = port
        self.ctx = ctx
        self.threads = threads
        self.n_gpu_layers = n_gpu_layers  # 99 offloads all layers (Vulkan); 0 = CPU
        self.proc = None
        self.log = None

    def start(self, log_path):
        self.log = open(log_path, "wb")
        self.proc = subprocess.Popen(
            [str(self.server_bin), "-m", str(self.model_path),
             "--host", "127.0.0.1", "--port", str(self.port),
             "-c", str(self.ctx), "-t", str(self.threads),
             "-ngl", str(self.n_gpu_layers)],
            stdout=self.log, stderr=subprocess.STDOUT,
            cwd=str(self.server_bin.parent))
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            assert self.proc.poll() is None, "llama-server exited during startup"
            try:
                with urllib.request.urlopen(
                        f"http://127.0.0.1:{self.port}/health", timeout=5) as r:
                    if json.loads(r.read().decode()).get("status") == "ok":
                        return
            except (OSError, ValueError):
                pass
            time.sleep(1.0)
        raise TimeoutError("llama-server /health not ok within 180s")

    def stop(self):
        if self.proc is not None and self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        self.proc = None
        if self.log is not None:
            self.log.close()
            self.log = None

    def props(self):
        with urllib.request.urlopen(
                f"http://127.0.0.1:{self.port}/props", timeout=10) as r:
            return json.loads(r.read().decode())


def chat(port, messages, seed, temperature, top_p=None, max_tokens=320,
         response_format=None, grammar=None, timeout_s=300):
    body = {"messages": messages, "seed": seed, "temperature": temperature,
            "max_tokens": max_tokens}
    if top_p is not None:
        body["top_p"] = top_p
    if response_format is not None:
        body["response_format"] = response_format
    if grammar is not None:
        body["grammar"] = grammar  # GBNF mask; a stage carries this XOR response_format
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/v1/chat/completions",
        data=json.dumps(body, ensure_ascii=True).encode(),
        headers={"Content-Type": "application/json"})
    t0 = time.monotonic()
    with urllib.request.urlopen(req, timeout=timeout_s) as r:
        resp = json.loads(r.read().decode())
    return {"request": body, "response": resp,
            "duration_ms": int((time.monotonic() - t0) * 1000)}
