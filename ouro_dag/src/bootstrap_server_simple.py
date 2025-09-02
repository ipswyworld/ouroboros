# bootstrap_server_simple.py
# Usage: python3 bootstrap_server_simple.py 8080 peers.txt

import http.server
import socketserver
import sys
import pathlib

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
PEERS_FILE = sys.argv[2] if len(sys.argv) > 2 else "peers.txt"

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path in ("/peers", "/peers.txt"):
            try:
                with open(PEERS_FILE, "r") as f:
                    data = f.read()
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode())
                return
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(data.encode())
        else:
            super().do_GET()

if __name__ == "__main__":
    print(f"Serving peers from {PEERS_FILE} on :{PORT}")
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        httpd.serve_forever()
