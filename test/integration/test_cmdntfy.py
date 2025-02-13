import os
import textwrap
from typing import Generator
import pytest
from pathlib import Path
import subprocess
from test_helper import TestHttpServer

BASE_PATH = Path(__file__).parents[2]


@pytest.fixture(scope="module")
def build_cmdntfy() -> Path:
    subprocess.check_call(["cargo", "build", "--release"], cwd=BASE_PATH)
    return Path(BASE_PATH, "target/release/cmdntfy")


@pytest.fixture()
def http_server() -> Generator[TestHttpServer, None, None]:
    server = TestHttpServer()
    server.start()
    yield server
    server.stop()


def test_notify_simple(build_cmdntfy: Path, http_server: TestHttpServer):
    """Test that cmdntfy can send a notification to a local HTTP server."""
    cmd: list[str | os.PathLike] = [
        build_cmdntfy,
        "--url",
        "http://localhost:8080",
        "--",
    ]
    cmd.extend(["echo", "hello"])
    proc = subprocess.run(
        cmd, stderr=subprocess.PIPE, stdout=subprocess.PIPE, text=True
    )

    assert proc.stdout == ""
    assert proc.stderr == ""
    assert proc.returncode == 0

    assert len(http_server.received_requests) == 1
    headers, body = http_server.received_requests[0]
    assert body == textwrap.dedent(
        """\
        stdout:
        
        ```
        hello

        ```

        stderr:

        ```

        ```"""
    )
    assert headers["x-markdown"] == "yes"
    assert headers["x-title"] == "Executing command echo"
    assert headers["x-priority"] == "low"
    assert headers["x-tags"] == "heavy_check_mark"
    assert "Authorization" not in headers
