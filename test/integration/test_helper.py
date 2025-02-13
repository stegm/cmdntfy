import http.server
import threading


class TestHttpServer:
    """A simple HTTP server that can be used for testing purposes."""

    def __init__(self):
        received_requests = []
        self.received_requests = received_requests

        class RequestHandler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                content_length = int(self.headers["content-length"])
                body = self.rfile.read(content_length).decode("utf-8")
                headers = dict(self.headers)

                received_requests.append((headers, body))

                self.send_response(200)
                self.end_headers()

        self.server = http.server.HTTPServer(("localhost", 8080), RequestHandler)

    def start(self):
        t = threading.Thread(target=self.server.serve_forever, daemon=True)
        t.start()

    def stop(self):
        self.server.shutdown()
        self.server.server_close()
